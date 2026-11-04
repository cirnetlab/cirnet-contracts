"""The route adapter must preserve the nonblocking gRPC Call/Future contract."""
from concurrent.futures import ThreadPoolExecutor, TimeoutError
import importlib
import threading
from time import monotonic, sleep

import grpc
import pytest


@pytest.fixture
def bindings(tmp_path, monkeypatch):
    from contracts.codegen.generate_hardware_rpc import generate
    output = tmp_path/'future_rpc'; generate(output)
    monkeypatch.syspath_prepend(str(tmp_path))
    return importlib.import_module('future_rpc.rpc_v2_pb2'), importlib.import_module('future_rpc.rpc_v2_pb2_grpc')


def server_for(pb, handlers):
    server = grpc.server(ThreadPoolExecutor(max_workers=8))
    for service, handler in handlers.items():
        method = grpc.unary_unary_rpc_method_handler(handler, request_deserializer=pb.Target.FromString,
                                                     response_serializer=pb.StationCapabilities.SerializeToString)
        server.add_generic_rpc_handlers((grpc.method_handlers_generic_handler(service, {'GetCapabilities': method}),))
    port = server.add_insecure_port('127.0.0.1:0'); server.start()
    return server, f'127.0.0.1:{port}'


@pytest.mark.parametrize('service', ['cirnet.hardware.v2.HardwareService', 'netcircuit.hardware.v2.HardwareService'])
def test_future_dispatch_is_nonblocking_for_delayed_canonical_and_legacy_rpc(bindings, service):
    pb, rpc = bindings
    release, entered, callback = threading.Event(), threading.Event(), threading.Event()
    seen = []
    def delayed(request, context):
        entered.set()
        release.wait(5)
        return pb.StationCapabilities(station_id=request.station_id)
    server, target = server_for(pb, {service: delayed})
    try:
        with grpc.insecure_channel(target) as channel, ThreadPoolExecutor(max_workers=1) as dispatcher:
            dispatch = dispatcher.submit(rpc.HardwareServiceStub(channel).GetCapabilities.future,
                                         pb.Target(station_id='delayed'), timeout=5)
            try:
                call = dispatch.result(timeout=.5)
                assert entered.wait(2)
                assert call.running() and not call.done()
                with pytest.raises(grpc.FutureTimeoutError): call.result(timeout=.01)
                assert call.running() and not call.done()
                def complete(future):
                    seen.append((future, future.result().station_id)); callback.set()
                call.add_done_callback(complete)
                release.set()
                assert call.result(timeout=2).station_id == 'delayed'
                assert callback.wait(2) and seen == [(call, 'delayed')]
                assert call.done() and not call.running() and not call.cancelled()
                late = []
                call.add_done_callback(lambda future: late.append(future))
                assert late == [call]
            finally: release.set()
    finally: release.set(); server.stop(0).wait()


def test_future_cancellation_is_immediate_and_never_starts_legacy_retry(bindings):
    pb, rpc = bindings
    release, entered, callback = threading.Event(), threading.Event(), threading.Event()
    seen, retries, call_callbacks = [], [], []
    def canonical(request, context):
        entered.set(); release.wait(5)
        context.abort(grpc.StatusCode.UNIMPLEMENTED, 'canonical unsupported')
    def legacy(request, context):
        retries.append(request.station_id)
        return pb.StationCapabilities(station_id=request.station_id)
    server, target = server_for(pb, {'cirnet.hardware.v2.HardwareService': canonical,
                                     'netcircuit.hardware.v2.HardwareService': legacy})
    try:
        with grpc.insecure_channel(target) as channel, ThreadPoolExecutor(max_workers=1) as dispatcher:
            dispatch = dispatcher.submit(rpc.HardwareServiceStub(channel).GetCapabilities.future, pb.Target(), timeout=5)
            try:
                call = dispatch.result(timeout=.5)
                assert entered.wait(2)
                call.add_done_callback(lambda future: (seen.append(future), callback.set()))
                assert call.add_callback(lambda: call_callbacks.append('done'))
                assert call.cancel() is True
                assert callback.wait(2)
                assert call.cancelled() and call.done() and not call.running()
                with pytest.raises(grpc.FutureCancelledError): call.result(timeout=1)
                assert call.code() == grpc.StatusCode.CANCELLED
                assert seen == [call] and call_callbacks == ['done']
                assert call.add_callback(lambda: call_callbacks.append('late')) is False
                assert call.cancel() is False
            finally: release.set()
        assert retries == []
    finally: release.set(); server.stop(0).wait()


def test_legacy_future_failure_and_with_call_keep_final_status_and_metadata(bindings):
    pb, rpc = bindings
    calls = []
    def legacy(request, context):
        calls.append(request.station_id)
        context.send_initial_metadata((('initial-key', 'legacy'),))
        context.set_trailing_metadata((('trailing-key', 'legacy'),))
        if request.station_id == 'denied': context.abort(grpc.StatusCode.PERMISSION_DENIED, 'legacy denied')
        return pb.StationCapabilities(station_id=request.station_id)
    server, target = server_for(pb, {'netcircuit.hardware.v2.HardwareService': legacy})
    try:
        with grpc.insecure_channel(target) as channel:
            method = rpc.HardwareServiceStub(channel).GetCapabilities
            value, call = method.with_call(pb.Target(station_id='ok'), timeout=2)
            assert value.station_id == 'ok' and call.code() == grpc.StatusCode.OK
            assert ('initial-key', 'legacy') in call.initial_metadata()
            assert ('trailing-key', 'legacy') in call.trailing_metadata()
            future = method.future(pb.Target(station_id='denied'), timeout=2)
            with pytest.raises(grpc.RpcError) as failure: future.result(timeout=2)
            assert failure.value.code() == future.code() == grpc.StatusCode.PERMISSION_DENIED
            assert future.details() == 'legacy denied'
            assert future.exception(timeout=0).code() == grpc.StatusCode.PERMISSION_DENIED
            assert ('trailing-key', 'legacy') in future.trailing_metadata()
        assert calls == ['ok', 'denied']
    finally: server.stop(0).wait()


def test_future_fallback_uses_one_total_deadline(bindings):
    pb, rpc = bindings
    attempts = []
    def canonical(request, context):
        attempts.append('canonical'); sleep(.08)
        context.abort(grpc.StatusCode.UNIMPLEMENTED, 'unsupported')
    def legacy(request, context):
        attempts.append('legacy'); sleep(.3)
        return pb.StationCapabilities()
    server, target = server_for(pb, {'cirnet.hardware.v2.HardwareService': canonical,
                                     'netcircuit.hardware.v2.HardwareService': legacy})
    try:
        with grpc.insecure_channel(target) as channel:
            started = monotonic()
            future = rpc.HardwareServiceStub(channel).GetCapabilities.future(pb.Target(), timeout=.18)
            with pytest.raises(grpc.RpcError) as failure: future.result(timeout=1)
            assert failure.value.code() == grpc.StatusCode.DEADLINE_EXCEEDED
            assert monotonic()-started < .3
            assert attempts == ['canonical', 'legacy']
    finally: server.stop(0).wait()
