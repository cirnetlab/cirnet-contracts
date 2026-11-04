"""Deterministic native-call boundary races for the gRPC route adapter."""
from concurrent.futures import Future, ThreadPoolExecutor, TimeoutError
import threading

import grpc
import pytest

from contracts.rpc_compat import _Details, _LegacyRouteFallback


class NativeCall:
    def __init__(self, metadata=()):
        self.future = Future()
        self.status = grpc.StatusCode.OK
        self.metadata = metadata
        self.cancel_calls = 0

    def add_done_callback(self, fn):
        self.future.add_done_callback(lambda _: fn(self))

    def complete(self, status=grpc.StatusCode.OK, result='result'):
        self.status = status
        self.future.set_result(result)

    def cancel(self):
        self.cancel_calls += 1
        return self.future.cancel()

    def done(self): return self.future.done()
    def code(self): return self.status
    def initial_metadata(self): return self.metadata
    def result(self): return self.future.result()


def pending_retry():
    primary = NativeCall((('route', 'unsupported'),))
    legacy = NativeCall((('route', 'legacy'),))
    creating, release = threading.Event(), threading.Event()
    def continuation(details, request):
        if details.method.startswith('/cirnet.'):
            return primary
        creating.set()
        assert release.wait(2)
        return legacy
    details = _Details('/cirnet.hardware.v2.HardwareService/GetCapabilities', 5, None, None, None, None)
    call = _LegacyRouteFallback().intercept_unary_unary(continuation, details, None)
    return primary, legacy, creating, release, call


def test_cancellation_during_retry_creation_cancels_the_new_native_call():
    primary, legacy, creating, release, call = pending_retry()
    callbacks = []
    call.add_done_callback(lambda future: callbacks.append(future))
    with ThreadPoolExecutor(max_workers=1) as executor:
        transition = executor.submit(primary.complete, grpc.StatusCode.UNIMPLEMENTED)
        try:
            assert creating.wait(1)
            assert call.cancel()
            assert call.done() and call.cancelled()
            with pytest.raises(grpc.FutureCancelledError): call.result(timeout=0)
            assert callbacks == [call]
        finally: release.set()
        transition.result(timeout=1)
    assert legacy.cancel_calls == 1 and legacy.future.cancelled()
    assert call.code() == grpc.StatusCode.CANCELLED and callbacks == [call]


def test_initial_metadata_waits_only_for_retry_selection_not_legacy_completion():
    primary, legacy, creating, release, call = pending_retry()
    with ThreadPoolExecutor(max_workers=2) as executor:
        transition = executor.submit(primary.complete, grpc.StatusCode.UNIMPLEMENTED)
        try:
            assert creating.wait(1)
            metadata = executor.submit(call.initial_metadata)
            with pytest.raises(TimeoutError): metadata.result(timeout=.02)
            release.set()
            assert metadata.result(timeout=1) == (('route', 'legacy'),)
            assert not call.done()
            legacy.complete(result='legacy result')
            assert call.result(timeout=1) == 'legacy result'
        finally: release.set()
        transition.result(timeout=1)


def test_initial_metadata_forwards_native_headers_without_waiting_for_result():
    primary = NativeCall((('ready', 'canonical'),))
    details = _Details('/cirnet.hardware.v2.HardwareService/GetCapabilities', 5, None, None, None, None)
    call = _LegacyRouteFallback().intercept_unary_unary(lambda details, request: primary, details, None)
    with ThreadPoolExecutor(max_workers=1) as executor:
        try:
            assert executor.submit(call.initial_metadata).result(timeout=.5) == (('ready', 'canonical'),)
            assert not call.done()
        finally: primary.complete()
