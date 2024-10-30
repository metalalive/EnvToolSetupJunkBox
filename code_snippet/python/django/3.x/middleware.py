
def db_middleware_exception_handler(func):
    # a decorator for handling database errors in the process of middleware
    from django.http import HttpResponse
    from django.db.utils import OperationalError
    from ecommerce_common.models.db import get_db_error_response

    def inner(self, *arg, **kwargs):
        try:
            response = func(self, *arg, **kwargs)
        except OperationalError as e:
            headers = {}
            status = get_db_error_response(e=e, headers=headers)
            # do NOT use DRF response since the request is being short-circuited by directly returning
            # custom response at here and it won't invoke subsequent (including view) middlewares.
            # Instead I use HttpResponse simply containing error response status without extra message
            # in the response body.
            response = HttpResponse(status=status)
            for k, v in headers.items():
                response[k] = v
            err_msg = " ".join(list(map(lambda x: str(x), e.args)))
            log_msg = ["status", status, "msg", err_msg]
            _logger.warning(None, *log_msg)
        return response

    return inner
