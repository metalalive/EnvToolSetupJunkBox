
```python
#EXTM3U
#EXT-X-VERSION:7
#EXT-X-TARGETDURATION:36
#EXT-X-MEDIA-SEQUENCE:0
#EXT-X-PLAYLIST-TYPE:VOD
#EXT-X-KEY:METHOD=AES-128,URI="http://your.key.server.com/key_file",IV=0x80152a1c871599356bd658617791d3d8
#EXT-X-MAP:URI="enc_init_packet_map"
#EXT-X-KEY:METHOD=AES-128,URI="http://your.key.server2.com/key_file",IV=0x80152a1c871599356bd658617791d3d8
#EXTINF:33.060000,
enc_seg_alpha
#EXTINF:30.240000,
enc_seg_beta
#EXTINF:34.620000,
enc_seg_circuit
#EXT-X-KEY:METHOD=AES-128,URI="http://your.key.server3.org/key_file",IV=0x8152a1c871599356bd658617791d3d81
#EXTINF:22.700000,
enc_seg_delta
#EXTINF:35.560000,
enc_seg_euler
#EXTINF:30.940000,
data_seg_formular
#EXTINF:24.060000,
data_seg_gamma
#EXTINF:29.000000,
data_seg_hypothesis
#EXTINF:30.880000,
data_seg_imaginary
#EXTINF:32.800000,
data_seg_journal
#EXTINF:0.073672,
data_seg_kick
#EXT-X-ENDLIST
```

Note:
* the URI attribute in  `EXT-X-KEY` tag should response with a key as a single packed array of 16 octets. For example, if you have hex string of the key, you can convert it to binary file `echo "6e572e12af942e78d9c2ab2bc8f137d8" | xxd -r -p > /path/to/key_file` then send the binary file `key_file` with http response.
* `IV` in `EXT-X-KEY` tag must start with `0x` since its value is treated as large number
*  to change key or IV in the middle, insert new line of `EXT-X-KEY` tag before the segment (starting with `#EXTINF`) you attempt to change
