import sys
from pathlib import Path

# Add package root to sys.path for pytest test discovery
sys.path.insert(0, str(Path(__file__).parent))
