import sys
from pathlib import Path

# Add package root to sys.path for pytest test discovery
# Add package root (one level up) to sys.path for pytest discovery
# Prepend the 'src' directory so tests import the package correctly
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))
