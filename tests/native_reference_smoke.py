"""Measured-fixture regression through the compiled shared library."""
import json
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT/'tools'))
from compare_native_motion import compare

result = compare(json.loads((ROOT/'tests/fixtures/native_r6_2026_10_06.json').read_text()))
assert abs(result['fitted_response_hz']-150)<0.001
for row in result['materials']:
    assert row['feedback']['velocity_rmse_studs_s']<0.002
    assert row['legacy']['velocity_rmse_studs_s']>4
for row in result['camera']:
    assert row['max_error_studs']<0.00001
jump=result['jump']
assert abs(jump['calibrated']['peak_rise_studs']-jump['native_peak_rise_studs'])<0.0001
assert jump['calibrated']['airborne_velocity_rmse_studs_s']<0.001
for row in result['independent_validation']:
    if row['label'].startswith(('friction_', 'walk_')):
        assert row['velocity_rmse_studs_s']<(0.7 if row['label']=='walk_32_1' else 0.002)
    elif row['label'] in ('power25','power50'):
        assert row['airborne_velocity_rmse_studs_s']<0.001
        assert abs(row['peak_rise_studs']-row['native_peak_rise_studs'])<0.0001
print('Recorded movement/camera checks passed; high-speed timing, power53, air steering and landing parity remain open.')
