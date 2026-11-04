"""Shared environment and concurrency settings for CIRNET services.

Canonical CIRNET_* variables take precedence, including explicit empty values.
NETCIRCUIT_* remains an input/output compatibility alias for older processes.
"""
import os
from collections.abc import Mapping


def getenv(name: str, default=None, environ: Mapping[str, str] | None = None):
    """Read canonical configuration with a legacy fallback without mutation."""
    environment = os.environ if environ is None else environ
    canonical = 'CIRNET_' + name[len('NETCIRCUIT_'):] if name.startswith('NETCIRCUIT_') else name
    if canonical in environment:
        return environment[canonical]
    if canonical.startswith('CIRNET_'):
        return environment.get('NETCIRCUIT_' + canonical[len('CIRNET_'):], default)
    return default


def export_environment(environ: Mapping[str, str] | None = None) -> dict[str, str]:
    """Copy the environment and synchronize aliases for child processes."""
    environment = dict(os.environ if environ is None else environ)
    suffixes = {name.split('_', 1)[1] for name in environment
                if name.startswith(('CIRNET_', 'NETCIRCUIT_'))}
    for suffix in suffixes:
        value = getenv('CIRNET_' + suffix, environ=environment)
        environment['CIRNET_' + suffix] = value
        environment['NETCIRCUIT_' + suffix] = value
    return environment


def simulation_worker_count(value: int | str | None = None) -> int:
    """Read or validate the positive simulation worker count, defaulting to four."""
    if value is None:
        value = getenv('CIRNET_SIMULATION_WORKERS', '4')
    if type(value) is int:
        workers = value
    elif isinstance(value, str) and value.strip().isascii() and value.strip().isdigit():
        workers = int(value.strip())
    else:
        raise ValueError('CIRNET_SIMULATION_WORKERS must be a positive integer')
    if workers <= 0:
        raise ValueError('CIRNET_SIMULATION_WORKERS must be a positive integer')
    return workers
