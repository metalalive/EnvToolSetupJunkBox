from django.core.files.storage import FileSystemStorage

class ExtendedFileSysStorage(FileSystemStorage):
    def __init__(
        self,
        location=None,
        base_url=None,
        file_permissions_mode=None,
        directory_permissions_mode=None,
        extra_id_required=None,
    ):
        """
        extra_id_required provides a list of ID fields required to generate appropriate
        renderred absolute path where a file is saved to, for example:
            TODO, finish this doc
        """
        self._extra_id_required = extra_id_required
        super().__init__(
            location=location,
            file_permissions_mode=file_permissions_mode,
            base_url=base_url,
            directory_permissions_mode=directory_permissions_mode,
        )

        # TODO, figure out how do I combine all these paths

    def get_valid_name(self, name):
        out = super().get_valid_name(name=name)
        return out

    def get_alternative_name(self, file_root, file_ext):
        out = super().get_alternative_name(file_root=file_root, file_ext=file_ext)
        return out

    def get_available_name(self, name, max_length=None):
        out = super().get_available_name(name=name, max_length=max_length)
        return out
