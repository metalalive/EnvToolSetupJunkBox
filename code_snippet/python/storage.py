import os as builtin_os  # built-in os package
import io as _io
import tempfile
import inspect
import asyncio
import copy
import secrets
import functools

from ecommerce_common.util.python import util_os


class AbstractStorage:
    """
    framework-independent abstract storage which defines interfaces so
    subclasses can inherit or overwrite them for specific platform.
    """

    def open(self, name, mode="rb", **kwargs):
        raise NotImplementedError("This backend doesn't support this function.")

    def _save_sanity_check(self, path, content):
        assert path, "path not specified"
        content_attr_check = (
            (content is not None)
            and hasattr(content, "read")
            and hasattr(content, "seek")
            and hasattr(content, "close")
            and hasattr(content, "tell")
        )
        if not content_attr_check:
            raise TypeError("the argument `content` lacks attributes")
        if hasattr(content, "closed"):
            assert (
                not content.closed
            ), "the input content should be a opened file-like object"

    def save(self, path, content, async_flg=False, **kwargs):
        """
        Save data bytes pointed by `content` to the file specified by `path`.
        The content should be any Python file-like object, ready to be read
        from the beginning.
        """
        self._save_sanity_check(path, content)
        if async_flg:
            coroutine = self._save_async(path, content, **kwargs)
            result = asyncio.run(coroutine)
        else:
            result = self._save(path, content, **kwargs)
        return result

    def _save(self, path, content, **kwargs):
        raise NotImplementedError("This backend doesn't support this function.")

    async def _save_async(self, path, content, **kwargs):
        raise NotImplementedError("This backend doesn't support this function.")

    def mkdir(self, path, allow_existing=False):
        """
        * `path` has to be absolute path in this storage platform
        * `allow_existing` indicates whether to reuse existing folder(s)
           while creating new directory to specific path. For example, an
           application invokes mkdir() with path `/path/to/abc123/def456/jkl789`
           , if `/path/to/abc123` already exists , and `allow_existing` is set to
           True, then it function call creates `def456/jkl789` at `/path/to/abc123`
        """
        raise NotImplementedError("This backend doesn't support this function.")

    def path(self, rel_path):
        """
        Return a local filesystem path where the file can be retrieved using
        Python's built-in open() function. Storage systems that can't be
        accessed using open() should *not* implement this method.
        """
        raise NotImplementedError("This backend doesn't support this function.")

    def get_alternative_path(self, path):
        dir_name, file_name = builtin_os.path.split(path)
        file_root, file_ext = builtin_os.path.splitext(file_name)
        while self.exists(path):
            alt_fname = self.get_alternative_filename(file_root, file_ext)
            path = builtin_os.path.join(dir_name, alt_fname)
        return path

    def get_alternative_filename(self, file_root, file_ext):
        """
        Default function to return an alternative filename, by adding an
        underscore and a random 7 character alphanumeric string (before the
        file extension, if one exists) to the filename.
        Sublcasses are free to overwrite this function
        """
        return "%s_%s%s" % (file_root, secrets.token_urlsafe(7), file_ext)

    def exists(self, name):
        """
        Return True if a file referenced by the given name already exists in the
        storage platform, or False if the name is available for a new file.
        """
        raise NotImplementedError("This backend doesn't support this function.")

    def delete(self, name):
        """
        Delete the specified file from the storage system.
        """
        raise NotImplementedError("This backend doesn't support this function.")


## end of class AbstractStorage


def _fs_storage_save_common(func):
    # generate sync or async decorator function based on whether the
    # input is asynchronous function or not.
    is_async_fn = inspect.isasyncgenfunction(
        func
    )  # don't use asyncio.iscoroutinefunction(func)

    @functools.wraps(func)  # TODO, figure out the way to refactor this function
    async def inner(
        self,
        path,
        content,
        chunk_sz=1024 * 4,
        non_file_types=None,
        alt_path_autogen=True,
    ):
        directory = builtin_os.path.dirname(path)
        self.mkdir(path=directory, allow_existing=True)
        full_path = self.path(rel_path=path)
        while True:
            # race condition happens when 2 threads invoke get_alternative_path()
            # concurrently then get the same path, such concurrency issue is handled
            # at low-level OS which raises FileExistsError
            try:
                if non_file_types:
                    all_non_file_types = copy.copy(self.non_file_types)
                    all_non_file_types.extend(non_file_types)
                else:
                    all_non_file_types = self.non_file_types
                # Typically `content` is a file object and its `name` attribute should
                # represents the path to the stored file, BUT this may NOT be true if
                # `content` is non-file types such as I/O stream (io.IOBase) , which
                # provides the same set of interfaces as built-in file object but it
                # does NOT have accessible file path for application callers.
                if getattr(content, "name", None) and not isinstance(
                    content, tuple(all_non_file_types)
                ):
                    util_os.safe_file_move(old_name=content.name, new_name=full_path)
                else:  # streaming at low level OS
                    _fd = builtin_os.open(full_path, self.OS_OPEN_FLAGS, 0o666)
                    _file = None
                    try:
                        util_os.fd_lock(_fd, util_os.LOCK_EX)
                        _generator = func(self, content, chunk_sz)
                        if is_async_fn:  # reset read pointer
                            await _generator.__anext__()
                        else:
                            next(_generator)
                        while True:
                            if (
                                is_async_fn
                            ):  # read chunk of data, or raise StopIteration
                                chunk = await _generator.__anext__()
                            else:
                                chunk = next(_generator)
                            if _file is None:
                                _mode = "wb" if isinstance(chunk, bytes) else "wt"
                                _file = builtin_os.fdopen(_fd, _mode)
                            if chunk:
                                _file.write(chunk)
                            else:
                                break
                    except (StopIteration, StopAsyncIteration):
                        pass
                    finally:
                        util_os.fd_unlock(_fd)
                        if _file is not None:
                            _file.close()
                        else:
                            builtin_os.close(_fd)
                    #### self._save_by_streaming(full_path, content, chunk_sz)
            except FileExistsError:
                if alt_path_autogen:
                    # generate an different name that hasn't been used if the file exists.
                    alt_path = self.get_alternative_path(path)
                    full_path = self.path(rel_path=alt_path)
                else:
                    raise
            else:  # try block goes well
                break
        return {"size": content.tell(), "path": full_path}

    ## end of inner
    @functools.wraps(func)
    def sync_wrapper_inner(
        self,
        path,
        content,
        chunk_sz=1024 * 4,
        alt_path_autogen=True,
        non_file_types=None,
    ):
        coroutine = inner(
            self=self,
            path=path,
            content=content,
            chunk_sz=chunk_sz,
            alt_path_autogen=alt_path_autogen,
            non_file_types=non_file_types,
        )
        return asyncio.run(coroutine)

    return inner if is_async_fn else sync_wrapper_inner


## end of _fs_storage_save_common()


class FileSystemStorage(AbstractStorage):
    OS_OPEN_FLAGS = (
        builtin_os.O_WRONLY
        | builtin_os.O_CREAT
        | builtin_os.O_EXCL
        | getattr(builtin_os, "O_BINARY", 0)
    )
    non_file_types = [_io.IOBase, tempfile.SpooledTemporaryFile]

    def __init__(self, location, directory_permissions_mode=None):
        # since this is framework-independent storage, this class will NOT check
        # and configure default settings
        assert builtin_os.path.exists(
            location
        ), "Base location of the file-system storage does NOT exist"
        acl_required = (
            builtin_os.R_OK | builtin_os.W_OK | builtin_os.X_OK | builtin_os.F_OK
        )
        assert builtin_os.access(
            location, acl_required
        ), "Current application does NOT have \
                access to the base location of the file-system"
        self._base_location = location
        self._directory_permissions_mode = directory_permissions_mode
        self._opened_files = []

    def path(self, rel_path):
        return util_os.safe_path_join(self._base_location, rel_path)

    def mkdir(self, path, allow_existing=False):
        full_path = self.path(rel_path=path)
        try:
            if self._directory_permissions_mode is not None:
                # builtin_os.makedirs applies the global umask, so we reset it,
                # for consistency with file_permissions_mode behavior.
                old_umask = builtin_os.umask(0)
                try:
                    builtin_os.makedirs(
                        full_path,
                        exist_ok=allow_existing,
                        mode=self._directory_permissions_mode,
                    )
                finally:
                    builtin_os.umask(old_umask)
            else:
                builtin_os.makedirs(full_path, exist_ok=allow_existing)
        except FileExistsError:
            raise FileExistsError("%s exists and is not a directory." % full_path)

    @_fs_storage_save_common
    async def _save_async(self, content, chunk_sz=1024 * 4):
        await content.seek(0)
        yield
        while True:
            chunk = await content.read(chunk_sz)
            if chunk:
                yield chunk
            else:
                break

    @_fs_storage_save_common
    def _save(self, content, chunk_sz=1024 * 4):
        content.seek(0)
        yield
        while True:
            chunk = content.read(chunk_sz)
            if chunk:
                yield chunk
            else:
                break

    def exists(self, name):
        full_path = self.path(rel_path=name)
        return builtin_os.path.exists(full_path)

    def delete(self, name):
        assert name, "The name argument is not allowed to be empty."
        full_path = self.path(rel_path=name)
        # If the file or directory exists, delete it from the filesystem.
        try:
            if builtin_os.path.isdir(full_path):
                builtin_os.rmdir(full_path)
            else:
                builtin_os.remove(full_path)
        except FileNotFoundError:
            # FileNotFoundError is raised if the file or directory was removed
            # concurrently.
            pass

    def open(self, path, mode="rb", manual_close=False):
        full_path = self.path(rel_path=path)
        f = open(full_path, mode)  # may raise FileNotFoundError
        f.manual_close = manual_close
        self._opened_files.append(f)
        return f

    def __del__(self):
        for f in self._opened_files:
            if f.manual_close:
                continue  # application caller will be responsible to close the file
            try:
                f.close()
            except FileNotFoundError:
                pass


## end of class FileSystemStorage
