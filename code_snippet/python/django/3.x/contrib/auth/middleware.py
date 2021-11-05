from datetime import datetime, timedelta, timezone
import math
import logging

from django.conf       import settings as django_settings
from django.core.cache       import caches as DjangoBuiltinCaches
from django.utils.http       import http_date
from django.utils.cache      import patch_vary_headers
from django.utils.module_loading import import_string
from django.contrib.auth.models  import AnonymousUser

from common.models.db import db_middleware_exception_handler
from common.auth.keystore import create_keystore_helper

_logger = logging.getLogger(__name__)


class JWTbaseMiddleware:
    DEFAULT_AUTH_BACKEND_INDEX = 0
    enable_setup = True
    enable_verify = True

    def __init__(self, get_response):
        self.get_response = get_response
        jwt_name_access_token  = getattr(django_settings, 'JWT_NAME_ACCESS_TOKEN', None)
        jwt_name_refresh_token = getattr(django_settings, 'JWT_NAME_REFRESH_TOKEN', None)
        access_token_valid_period = getattr(django_settings, 'JWT_ACCESS_TOKEN_VALID_PERIOD', 0)
        err_msg = 'all of the parameters have to be set when applying JWTbaseMiddleware , but some of them are unconfigured, JWT_NAME_ACCESS_TOKEN = %s , JWT_NAME_REFRESH_TOKEN = %s , JWT_ACCESS_TOKEN_VALID_PERIOD = %s'
        err_msg = err_msg % (jwt_name_access_token, jwt_name_refresh_token, access_token_valid_period)
        assert jwt_name_access_token and jwt_name_refresh_token and access_token_valid_period \
                and access_token_valid_period > 0, err_msg
        self._backend_map = {}
        cnt = self.DEFAULT_AUTH_BACKEND_INDEX
        for bkn_path in django_settings.AUTHENTICATION_BACKENDS:
            self._backend_map[cnt] = bkn_path
            self._backend_map[bkn_path] = cnt
            cnt += 1
        # initialize keystore, with associated persistence handlers (separate for fetching secret and public key)
        self._keystore = create_keystore_helper(cfg=django_settings.AUTH_KEYSTORE, import_fn=import_string)

    @db_middleware_exception_handler
    def __call__(self, request):
        if self.enable_verify :
            self.process_request(request)
        response = self.get_response(request)
        if self.enable_setup :
            self.process_response(request, response)
        return response

    def process_response(self, request, response):
        if hasattr(request, 'jwt'):
            if request.jwt.destroy:
                self._remove_token(request, response)
            else:
                self._set_tokens_to_cookie(request, response)

    def _remove_token(self, request, response):
        response.delete_cookie(
            django_settings.JWT_NAME_ACCESS_TOKEN, domain=None,
            path=django_settings.SESSION_COOKIE_PATH,
        )
        response.delete_cookie(
            django_settings.JWT_NAME_REFRESH_TOKEN, domain=None,
            path=django_settings.SESSION_COOKIE_PATH,
        ) # TODO, delete tokens generated for remote services
        patch_vary_headers(response, ('Cookie',))

    def _set_tokens_to_cookie(self, request, response):
        """
        For traditional web frontend where clients switch between web pages,
        it is better to store refresh/access tokens to client's cookie
        (with http-only flag set)
        For SPA (stands for single page application) no need to store
        the tokens to client's cookie , instead the authentication server
        simply store new (access) token within response body of POST request
        , so frontend extracts the tokens from the response and keep it
        in memory. (TODO: figure out if there is any security issue)
        """
        for entry in request.jwt.entries:
            self._set_token_to_cookie(request, response, **entry)

    def _set_token_to_cookie(self, request, response, jwtobj, cookie_name,
            max_age, cookie_domain):
        if jwtobj is None :
            return
        if not jwtobj.modified:
            log_args = ['cookie_name', cookie_name, 'msg', 'this token has not been modified']
            _logger.debug(None, *log_args)
            return
        # encode backend module path to index
        default_backend_path = self._backend_map[self.DEFAULT_AUTH_BACKEND_INDEX]
        backend_path = jwtobj.payload.get('bkn_id', default_backend_path)
        if isinstance(backend_path, str):
            jwtobj.payload['bkn_id'] = self._backend_map[backend_path]
        # defaults some claims in header & payload section,
        issued_at = datetime.utcnow()
        expires = issued_at + timedelta(seconds=max_age)
        # exp & iat field must be NumericDate, see section 4.1.4 , RFC7519
        default_payld = {'exp':  expires, 'iat': issued_at, 'iss':'YOUR_ISSUER'}
        default_header = {}
        jwtobj.default_claims(header_kwargs=default_header, payld_kwargs=default_payld)
        # then encode & sign the token , using private key (secret) provided by the keystore
        encoded = jwtobj.encode(keystore=self._keystore)
        response.set_cookie(
            key=cookie_name, value=encoded,  max_age=max_age,
            expires=http_date(expires.timestamp()),  domain=cookie_domain,
            path=django_settings.SESSION_COOKIE_PATH,
            secure=django_settings.SESSION_COOKIE_SECURE or None,
            samesite=django_settings.SESSION_COOKIE_SAMESITE,
            httponly=True
        )


    def process_request(self, request):
        user = None
        request._keystore  = self._keystore
        payld = jwt_httpreq_verify(request=request)
        if payld: # and request.jwt.acs.valid is True
            acc_id = payld.get('acc_id', None)
            backend_id = payld.get('bkn_id', self.DEFAULT_AUTH_BACKEND_INDEX)
            user = self._get_user(acc_id, backend_id)
            request.jwt.user = user
        request.user = user or AnonymousUser()

    def _get_user(self, acc_id, backend_id):
        """
        since django auth.get_user() is tied closely with session object,
        the get_user() cannot be applied after JWT authentication.

        load backend class for manipulating user model
        """
        assert backend_id is not None and backend_id >= self.DEFAULT_AUTH_BACKEND_INDEX, 'backend_id must not be null'
        backend_path = self._backend_map[backend_id] # decode index to module path
        backend = import_string(backend_path)()
        return backend.get_user(acc_id)

## end of class JWTbaseMiddleware


class JWTsetupMiddleware(JWTbaseMiddleware):
    enable_setup = True
    enable_verify = False

class JWTverifyMiddleware(JWTbaseMiddleware):
    enable_setup = False
    enable_verify = True


