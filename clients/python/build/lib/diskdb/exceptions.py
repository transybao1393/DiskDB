"""
DiskDB Exception Classes
"""


class DiskDBError(Exception):
    """Base exception for all DiskDB errors."""
    pass


class ConnectionError(DiskDBError):
    """Raised when connection to DiskDB server fails."""
    pass


class CommandError(DiskDBError):
    """Raised when a command execution fails."""
    pass


class TypeMismatchError(CommandError):
    """Raised when operation is performed on wrong data type."""
    pass


class TimeoutError(DiskDBError):
    """Raised when operation times out."""
    pass