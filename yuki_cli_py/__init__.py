"""
yuki-cli: CLI for Nmbrs Accounting (formerly Yuki) SOAP API.
"""

try:
    from importlib.metadata import version
    __version__ = version("yuki-cli")
except ImportError:
    from importlib_metadata import version
    __version__ = version("yuki-cli")
