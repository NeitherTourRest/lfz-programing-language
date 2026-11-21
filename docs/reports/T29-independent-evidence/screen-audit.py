"""Archived offline entry, checks both screen and frozen gate, stdout by default."""
from pathlib import Path
import runpy
runpy.run_path(str(Path(__file__).with_name('offline-audit.py')),run_name='__main__')
