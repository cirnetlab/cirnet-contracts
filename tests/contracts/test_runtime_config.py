"""Validate the shared simulation worker count at its Contracts boundary."""

import pytest

from contracts.runtime_config import simulation_worker_count


@pytest.mark.parametrize('value,expected', [(None, 4), ('1', 1), ('8', 8)])
def test_shared_worker_setting_defaults_and_reads_environment(monkeypatch, value, expected):
    monkeypatch.delenv('CIRNET_SIMULATION_WORKERS', raising=False)
    monkeypatch.delenv('NETCIRCUIT_SIMULATION_WORKERS', raising=False)
    if value is not None:
        monkeypatch.setenv('CIRNET_SIMULATION_WORKERS', value)
    assert simulation_worker_count() == expected


@pytest.mark.parametrize('value', [0, -1, True, 1.5, '', '0', '-2', '1.5', 'many'])
def test_shared_worker_setting_rejects_invalid_explicit_values(value):
    with pytest.raises(ValueError, match='CIRNET_SIMULATION_WORKERS'):
        simulation_worker_count(value)
