#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <uv.h>

typedef struct {
    uv_work_t *reqs;
    int num;
} sig_ctx_t;

typedef struct {
    uv_signal_t *sig;
    int  fib_n;
    int *num_req_done;
    int  max_num_reqs;
} shr_ctx_t;

static int _fib(int n) {
    if(n == 0 || n == 1) {
        return 1;
    } else {
        return _fib(n - 1) + _fib(n - 2);
    }
}

static void fib(uv_work_t *req) {
    shr_ctx_t *ctx = (shr_ctx_t *)req->data;
    int n = ctx->fib_n;
    if(random() % 2) {
        sleep(1);
    } else {
        sleep(3);
    }
    int sum = _fib(n);
    fprintf(stderr, "%dth fibonacci is %d \n", n, sum);
}

static void after_fib(uv_work_t *req, int status) {
    shr_ctx_t *ctx = (shr_ctx_t *)req->data;
    int n = ctx->fib_n;
    if (status == UV_ECANCELED) {
        fprintf(stderr, "Canceled %dth fibonacci \n", n);
    } else {
        (*ctx->num_req_done)++;
        int num_req_done = *ctx->num_req_done;
        assert(ctx->max_num_reqs >= num_req_done);
        if(ctx->max_num_reqs == num_req_done) {
            uv_signal_stop(ctx->sig);
            uv_close((uv_handle_t *)ctx->sig, NULL);
        }
        fprintf(stderr, "Done calculating %dth fibonacci, number of requests done: %d \n", n, num_req_done);
    }
}

static void sig_int_handler(uv_signal_t *req, int signum) {
    fprintf(stdout, "Signal received (%d) \n", signum);
    int idx = 0;
    sig_ctx_t  *ctx = (sig_ctx_t *)req->data;
    // TODO, figure out why not all pending requests are cancelled successfully,
    //  also figure out why memory leak happens to the requests.
    for(idx = 0; idx < ctx->num; idx++) {
        uv_work_t curr_req = ctx->reqs[idx];
        uv_cancel((uv_req_t *)&curr_req);
    }
    uv_signal_stop(req);
    uv_close((uv_handle_t *)req, NULL);
}


int main(int argc, char *argv[]) {
    assert(argc >= 1);
    int fib_until_idx = argc - 1;
    int fib_until = (int) strtol(argv[fib_until_idx], (char **)NULL, 10);
    assert(fib_until <= 100);
    uv_loop_t *loop = uv_default_loop();

    shr_ctx_t data[fib_until];
    uv_work_t req[fib_until];
    uv_signal_t sig;
    int result = 0;
    int idx = 0;
    int num_req_done = 0;

    for(idx = 0; idx < fib_until; idx++) {
        data[idx].fib_n = idx;
        data[idx].num_req_done = &num_req_done;
        data[idx].max_num_reqs =  fib_until;
        data[idx].sig = &sig;
        req[idx].data = (void *)&data[idx];
        // run the blocking tasks in separate threads, it seems that libuv manages to create number of threads, it is not like `one thread for each task`.
        uv_queue_work(loop, &req[idx], fib, after_fib);
    }

    sig_ctx_t  sig_ctx = {.reqs = &req[0], .num = fib_until};
    sig.data = (void *)&sig_ctx;
    uv_signal_init(loop, &sig);
    uv_signal_start(&sig, sig_int_handler, SIGINT);

    uv_run(loop, UV_RUN_DEFAULT);

    //uv_close((uv_handle_t *)&sig, NULL);
    result = uv_loop_close(loop);
    if(result != 0) {
        fprintf(stderr, "failed to close loop, reason: %s \n", uv_strerror(result));
    }
    return 0;
}
