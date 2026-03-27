"""
yuki-cli: CLI client for the Yuki bookkeeping SOAP API.
"""

try:
    from importlib.metadata import version
    __version__ = version("yuki-cli")
except ImportError:
    from importlib_metadata import version
    __version__ = version("yuki-cli")
