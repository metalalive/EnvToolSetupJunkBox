#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <uv.h>

typedef struct {
    uv_async_t *async;
    int total_size;
    float percentage;
} context_t;

static void update_progress(uv_work_t *req) {
    context_t *ctx = (context_t *)req->data;
    uv_async_t *async = ctx->async;
    int total_size = ctx->total_size;
    int processed = 0;
    while(processed < total_size) {
        ctx->percentage = processed * 100.f / total_size;
        // this example has only one request so I don't use mutex / rwlock to protect `data` field (of uv_work_t) , for multi-threaded case you should ensure the accesses are performed in right order by applying mutex / rwlock on each uv_work_t request.
        async->data = (void *)&ctx->percentage;
        uv_async_send(async); // only meant to wake up event loop
        usleep((useconds_t)10000);
        processed += (100 + random()) % 500; // simulate variant processing speed
    }
    // clean up the watcher, can also be asynchronous
    uv_close((uv_handle_t *)async, NULL);
}

static void print_progress(uv_async_t *handle) {
    float _percentage = *(float *)handle->data ;
    fprintf(stdout, "processed %.2f%% \n", _percentage);
}

int main(int argc, char *argv[]) {
    int total_size = 10240;
    int result = 0;
    uv_async_t async;
    uv_work_t  req;
    uv_loop_t *loop = uv_default_loop();

    context_t ctx = {.async = &async, .total_size = total_size, 
        .percentage = 0.0f };
    req.data = (void *)&ctx;
    uv_async_init(loop, &async, print_progress);
    uv_queue_work(loop, &req, update_progress, NULL);
    uv_run(loop, UV_RUN_DEFAULT);

    result = uv_loop_close(loop);
    if(result != 0) {
        fprintf(stderr, "failed to close loop, reason: %s \n", uv_strerror(result));
    }
    return 0;
}
