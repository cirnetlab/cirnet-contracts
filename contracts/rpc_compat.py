"""Legacy gRPC route adapters; protobuf fields retain their existing wire bytes."""
from collections import namedtuple
from concurrent.futures import CancelledError, Future, TimeoutError
from threading import Condition, RLock
from time import monotonic

import grpc


_Details = namedtuple('_Details', 'method timeout metadata credentials wait_for_ready compression')


class _RouteFallbackCall(grpc.Future, grpc.Call):
    """Select one terminal call without waiting in the interceptor dispatch path.

    A completion callback starts the legacy attempt. The lock arbitrates native
    completion, cancellation, and a retry being created outside the lock; user
    callbacks always run after the lock is released.
    """
    def __init__(self, primary, retry, deadline):
        self._lock = RLock()
        self._changed = Condition(self._lock)
        self._primary = primary
        self._completion = Future()
        self._active = primary
        self._retry = retry
        self._deadline = deadline
        self._terminal = False
        self._retry_started = False
        primary.add_done_callback(self._primary_done)

    def _primary_done(self, call):
        with self._lock:
            if self._terminal or self._retry_started:
                return
        if call.code() != grpc.StatusCode.UNIMPLEMENTED:
            self._finish(call)
            return
        with self._lock:
            if self._terminal or self._retry_started:
                return
            self._retry_started = True
        # Do not hold the lock while invoking another interceptor/native channel.
        # A concurrent cancel must return even while that invocation is starting.
        try:
            legacy = self._retry()
        except Exception as error:
            with self._lock:
                if self._terminal:
                    return
                self._terminal = True
                self._changed.notify_all()
            self._completion.set_exception(error)
            return
        with self._lock:
            cancelled = self._terminal
            if not cancelled:
                self._active = legacy
            self._changed.notify_all()
        if cancelled:
            legacy.cancel()
        else:
            legacy.add_done_callback(self._finish)

    def _finish(self, call):
        with self._lock:
            if self._terminal or call is not self._active:
                return
            self._terminal = True
            self._changed.notify_all()
        self._completion.set_result(call)

    def cancel(self):
        with self._lock:
            if self._terminal:
                return False
            self._terminal = True
            active = self._active
            self._changed.notify_all()
        # Marking terminal first prevents a native cancellation callback from
        # starting a retry. A retry already being created is cancelled on arrival.
        active.cancel()
        self._completion.cancel()
        return True

    def cancelled(self):
        return self._completion.cancelled()

    def done(self):
        return self._completion.done()

    def running(self):
        return not self.done()

    def is_active(self):
        return not self.done()

    def time_remaining(self):
        return None if self._deadline is None else max(0, self._deadline-monotonic())

    def _wait(self, timeout=None):
        try:
            return self._completion.result(timeout)
        except CancelledError:
            raise grpc.FutureCancelledError() from None
        except TimeoutError:
            raise grpc.FutureTimeoutError() from None

    def result(self, timeout=None):
        return self._wait(timeout).result()

    def exception(self, timeout=None):
        try:
            call = self._wait(timeout)
        except (grpc.FutureTimeoutError, grpc.FutureCancelledError):
            raise
        except Exception as error:
            return error
        return call.exception()

    def traceback(self, timeout=None):
        try:
            call = self._wait(timeout)
        except (grpc.FutureTimeoutError, grpc.FutureCancelledError):
            raise
        except Exception as error:
            return error.__traceback__
        return call.traceback()

    def add_done_callback(self, fn):
        self._completion.add_done_callback(lambda _: fn(self))

    def add_callback(self, callback):
        with self._lock:
            if self._terminal:
                return False
            self._completion.add_done_callback(lambda _: callback())
            return True

    def _call_value(self, method, cancelled_value=None, failed_value=None):
        try:
            call = self._wait()
        except grpc.FutureCancelledError:
            return cancelled_value
        except Exception:
            return failed_value
        return getattr(call, method)()

    def initial_metadata(self):
        while True:
            with self._changed:
                if self._terminal:
                    break
                active = self._active
                if self._retry_started and active is self._primary:
                    self._changed.wait()
                    continue
            metadata = active.initial_metadata()
            with self._changed:
                if self._terminal:
                    break
                if active is not self._active:
                    continue
                # A completed unsupported route is about to be replaced; wait
                # for that selection, rather than leaking its empty headers.
                if active is self._primary and active.done() and active.code() == grpc.StatusCode.UNIMPLEMENTED:
                    self._changed.wait()
                    continue
                return metadata
        return self._call_value('initial_metadata')

    def trailing_metadata(self):
        return self._call_value('trailing_metadata')

    def code(self):
        return self._call_value('code', grpc.StatusCode.CANCELLED, grpc.StatusCode.INTERNAL)

    def details(self):
        return self._call_value('details', 'Locally cancelled by application!',
                                'Exception raised while intercepting the RPC')


class _LegacyRouteFallback(grpc.UnaryUnaryClientInterceptor):
    def intercept_unary_unary(self, continuation, details, request):
        started = monotonic()
        legacy = details.method.replace('/cirnet.hardware.v2.', '/netcircuit.hardware.v2.', 1)
        primary = continuation(details, request)
        if legacy == details.method:
            return primary
        deadline = None if details.timeout is None else started + details.timeout
        def retry():
            timeout = None if deadline is None else max(0, deadline-monotonic())
            return continuation(_Details(legacy, timeout, details.metadata, details.credentials,
                                         details.wait_for_ready, details.compression), request)
        return _RouteFallbackCall(primary, retry, deadline)


def compatibility_channel(channel):
    """Try the canonical route first and retry only an unsupported legacy route."""
    return grpc.intercept_channel(channel, _LegacyRouteFallback())
