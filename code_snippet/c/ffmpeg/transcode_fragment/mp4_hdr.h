#ifndef MP4_HDR_H
#define MP4_HDR_H
#ifdef __cplusplus
extern "C" {
#endif

int  serialize_mp4_header(int dst_fd, int src_fd);

#ifdef __cplusplus
} // end of extern C clause
#endif
#endif // end of MP4_HDR_H
