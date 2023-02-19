### Certificate setup

#### generate CA cert / (RSA) private key
```
openssl genrsa -out ca_priv_key.pem  2048

openssl req -new -x509 -days 180 -key ca_priv_key.pem -keyform PEM -out ca_crt.pem -outform PEM -sha384

openssl x509 -text -noout -in ca_crt.pem -inform PEM
```

#### generate RSA public key derived from any given RSA private key
```
openssl rsa  -in  ca_priv_key.pem  -outform PEM -pubout -out rsa_pubkey.pem
```

#### generate server cert, sign it using CA cert 

* x509v3 extension can be added by the option `extensions v3_req` and `-extfile <OPENSSL_INSTALL_PATH>/openssl.cnf`
* signature size is determined by RSA key kength (e.g. 2048 bits in the command below)
```
openssl genrsa -out server_priv_key.pem  2048

openssl req -new  -days 1 -key server_priv_key.pem -keyform PEM -out server_csr.pem -outform PEM  -sha384

// edit subjectAltName=IP:xxx.xxx.xxx.xxx at usr_cert section of /usr/local/ssl/openssl.cnf, then run the command below
openssl x509 -req -in server_csr.pem -extfile /usr/local/ssl/openssl.cnf -extensions v3_req  usr_cert  \
    -CA ca_crt.pem -CAform PEM -CAkey ca_priv_key.pem -CAkeyform PEM -CAcreateserial -out server_crt.pem \
    -outform PEM -days 1 -sha384
```
[Some languages (e.g. Python >= 3.7) do NOT support IP address in CN field when decoding x509 certificate](https://stackoverflow.com/questions/52855924/problems-using-paho-mqtt-client-with-python-3-7) , so it will be treated as error. To avoid this error `subjectAltName=IP:xxx.xxx.xxx.xxx` has to be added to the server certificate file.




#### Read generated certificate
```
openssl x509 -text -noout -in server_crt.pem -inform PEM
```

#### Verify the certicate signed by the CA
```
openssl verify -verbose -CAfile /PATH/TO/ca.crt  /PATH/TO/whatever_cert_to_test.crt 
```
You should see the result if it is verified successfully
```
> /PATH/TO/whatever_cert_to_test.crt: OK
```


#### for extracting signature bit string 
```
openssl x509 -text -noout -in ca_crt_test.pem -certopt ca_default -certopt no_serial -certopt no_subject -certopt no_validity -certopt no_extensions -certopt no_signame |  grep -v 'Signature Algorithm' | tr -d '[:space:]' | xxd -r -p > ca_crt_test_sig.bin

xxd ca_crt_test_sig.bin
```

#### for extracting public key from x509 certificate
```
openssl x509 -in ca_crt_test.pem -noout -pubkey > ca_pubkey_test.pem
```

#### for decrypted / unpadding signature
```
openssl rsautl -verify -inkey ca_pubkey_test.pem -in ca_crt_test_sig.bin  -pubin  > ca_crt_test_sig_decrypted.bin

xxd ca_crt_test_sig_decrypted.bin
```

### AES encryption
* encryption using AES 128-bit with CBC (cipher-block chaining)
```
openssl  aes-128-cbc  -e -K e572e12af942e78d9c2ab2bc8f137d86  -iv 152a1c871599356bd658617791d3d801 \
      -in  /path/to/origin_file  -out /path/to/processed_file
```
* to decrypt the encrypted file, replace `-e` with `-d`
* The raw key option `-K` indicates hex string of 16 octets (16 x 8 = 128 bits)
* `-iv` (initialization vector) is required if `-K` is specified

### TLS connection
OpenSSL provides a test utility `s_client` acting as frontend client initiating TLS handshake to any server. The examples in this section demonstrates how `s_client` works in TLS v1.3 protocol.

#### The first handshake
```shell
openssl s_client -CAfile /PATH/TO/CERTS/CA.crt   -tls1_3 -state  -sess_out /PATH/TO/OUTPUT/SESSION-PARAM.pem \
    -keylogfile=/PATH/TO/key-log-file.txt  -connect localhost:1234
```
Note:
- Certificate specified by `-CAfile` is usuaully required if there's no pre-shared key specified, it can also specify path to self-signed CA.
- `-keylogfile` indicates path to a [key log](https://firefox-source-docs.mozilla.org/security/nss/legacy/key_log_format/index.html) file which records per-session secrets (pre-master secrets) for other analysis tools, for example, debug and decrypt TLS traffic in [wireshark](https://en.wikipedia.org/wiki/Wireshark).
  - also note this option has been introduced since openssl 1.1.1, does not work in older version.
- if the handshake is done successfully, `-sess_out` can be used to save the session, in case the server responds with a post-handshake message named  [`NewSessionTicket`](https://www.rfc-editor.org/rfc/rfc8446#section-4.6.1) (also encrypted).
  - the content of the saved session file depends on your TLS version, for example in TLS 1.3, it includes pre-shared key / ticket data. See [View a saved session file](#view-a-saved-session-file) below.
- `-brief` option can also be added, to reduce logging message printed (e.g. certificate and ssl-session data, see below).

The possible logging message would be :
```shell
CONNECTED(00000005)
SSL_connect:before SSL initialization
SSL_connect:SSLv3/TLS write client hello
SSL_connect:SSLv3/TLS write client hello
SSL_connect:SSLv3/TLS read server hello
SSL_connect:TLSv1.3 read encrypted extensions
depth=1 C = TW, O = CA organization, CN = app_tester_ca
verify return:1
depth=0 C = SG, O = Service Provider, CN = app_tester
verify return:1
SSL_connect:SSLv3/TLS read server certificate
SSL_connect:TLSv1.3 read server certificate verify
SSL_connect:SSLv3/TLS read finished
SSL_connect:SSLv3/TLS write change cipher spec
SSL_connect:SSLv3/TLS write finished
---
Certificate chain
 0 s:C = SG, O = Service Provider, CN = app_tester
   i:C = TW, O = CA organization, CN = app_tester_ca
---
Server certificate
-----BEGIN CERTIFICATE-----
... omit unimportant detail ...
-----END CERTIFICATE-----
subject=C = SG, O = Service Provider, CN = app_tester
issuer=C = TW, O = CA organization, CN = app_tester_ca

---
No client certificate CA names sent
Peer signing digest: SHA256
Peer signature type: RSA-PSS
Server Temp Key: X25519, 253 bits
---
SSL handshake has read 1651 bytes and written 295 bytes
Verification: OK
---
New, TLSv1.3, Cipher is TLS_AES_256_GCM_SHA384
Server public key is 2048 bit
Secure Renegotiation IS NOT supported
Compression: NONE
Expansion: NONE
No ALPN negotiated
Early data was not sent
Verify return code: 0 (ok)
---
---
Post-Handshake New Session Ticket arrived:
SSL-Session:
    Protocol  : TLSv1.3
    Cipher    : TLS_AES_256_GCM_SHA384
    Session-ID: 2CE5D25B9744186717D1C052301F9BF847D8ACC58CBA6F9BC0C64C7298A57844
    Session-ID-ctx: 
    Resumption PSK: 4FA12AE6338ADAEE76961DF2007E842EB8887A2F88577D6FD41EDE491C99B53A834557EC37E83894ABA3BC6A894735B2
    PSK identity: None
    PSK identity hint: None
    SRP username: None
    TLS session ticket lifetime hint: 420 (seconds)
    TLS session ticket:
        0000 - 8d f5 bc 45 e2 43 f5 c8-65 8b 18 ca b6 b8 ae bf   ...E.C..e.......
        0010 - 0d c4 df c2 45 2a e6 a4-19 c7 04 d9 5f 79 1a 03   ....E*......_y..
        0020 - d7 16 b9 ce 3e f8 21 34-45 06 49 cf ec 54 80 7a   ....>.!4E.I..T.z
        0030 - ee e5 96 e7 80 9d f6 5f-14 56 ae cb f9 40 3f 54   ......._.V...@?T
        .... omit unimportant data ....
---
SSL_connect:SSLv3/TLS read server session ticket
read R BLOCK
SSL_connect:SSL negotiation finished successfully
```
Note:
- the first time is usually a full TLS handshake, as shown in the logging message above, server sends `ServerHello`, `Certificate`, `CertificateVerify`, and `Finish` at the end of the handshake, and client has to verify server's certificate (see [Figure 1 of section 2, RFC8446](https://www.rfc-editor.org/rfc/rfc8446#section-2))
- the line `Verification: OK` and `Verify return code: 0 (ok)` indicates the result of certificate verification
- the line `New, TLSv1.3, Cipher is XXX` indicates current handshake will create a new session
  - **TODO**, figure out what new session means, does openssl internally maintain valid TLS sessions for later reuse ?
- the line `Post-Handshake xxx` means the server might send other messages after handshake is done successfully , in this example, it is `NewSessionTicket`

#### Subsequent handshakes
once the session file is saved, you can reuse it for subsequent TLS handshakes to the same host/port by the expiry time
```shell
openssl  s_client -noservername -sess_in /PATH/TO/OUTPUT/SESSION-PARAM.pem \
     -sess_out  /PATH/TO/OUTPUT/SESSION-PARAM.pem \
     -keylogfile=/PATH/TO/key-log-file.txt  -tls1_3  -state  -connect localhost:1234
```
Note
- `s_client` read saved session file from the path specified by `-sess_in`
- `-sess_out` and `-sess_in` can point to the same path, the saved session file will be overwritten if the handshake is done successfully.

The logging message would be :
```shell
CONNECTED(00000005)
SSL_connect:before SSL initialization
SSL_connect:SSLv3/TLS write client hello
SSL_connect:SSLv3/TLS write client hello
SSL_connect:SSLv3/TLS read server hello
SSL_connect:TLSv1.3 read encrypted extensions
SSL_connect:SSLv3/TLS read finished
SSL_connect:SSLv3/TLS write change cipher spec
SSL_connect:SSLv3/TLS write finished
..... omit because nothing changed .......  
---
SSL handshake has read 241 bytes and written 582 bytes
Verification: OK
---
Reused, TLSv1.3, Cipher is TLS_AES_256_GCM_SHA384
Server public key is 2048 bit
Secure Renegotiation IS NOT supported
..... omit because nothing changed .......  
```
Note:
- it is as similar as the first handshake, except it doesn't have to go through a full handshake.
  - if the server is able to recognize the pre-shared key in the session file `/PATH/TO/OUTPUT/SESSION-PARAM.pem`, it will skip certificate verification flow and go straight to `EncryptedExtensions` and final `Finished` message.
- the line `Reused, TLSv1.3, Cipher is XXX` indicates current handshake resumes previous TLS session.
  - **TODO**, does openssl internally maintain valid TLS sessions for later reuse ?


### View a saved session file
To dump detail in a saved session file, use subcommand `sess_id`
```shell
openssl sess_id -in  /PATH/TO/OUTPUT/SESSION-PARAM.pem -text -noout
```
The format of the detail looks like following :
```shell
SSL-Session:
    Protocol  : TLSv1.3
    Cipher    : TLS_AES_256_GCM_SHA384
    Session-ID: 46A0F9AA7121168807CDBCDCE4CD49AADCBEC567D1DF2A5857E0B16F66649429
    Session-ID-ctx: 
    Resumption PSK: 82A5CFE907CFBAF40468901CC6F18AEA829DE3C6D1299FD011AEF9498A111546CA41CC1B4F36A473D3A5953884BF687D
    PSK identity: None
    PSK identity hint: None
    SRP username: None
    TLS session ticket lifetime hint: 420 (seconds)
    TLS session ticket:
    0000 - bc 3a 21 03 c3 13 c2 15-6f 08 a5 cb 2e f4 95 68   .:!.....o......h
    0010 - 53 b4 ef 5c 34 46 b4 f1-5c 1b f3 dc 3a ca 4b 81   S..\4F..\...:.K.
    0020 - 73 62 cb 34 86 97 33 28-f1 3f 5e 56 9b a8 e7 03   sb.4..3(.?^V....
    0030 - c3 3b 44 8e 21 a4 ce b3-70 21 47 b9 ac da 02 ff   .;D.!...p!G.....
    ......... << omit rest of bytes >> ........

    Start Time: 1676815357
    Timeout   : 7200 (sec)
    Verify return code: 0 (ok)
    Extended master secret: no
    Max Early Data: 0
```
Note:
- the concept of *session ID* and *session tickets(RFC5077)* is obsolete in TLS 1.3
  - the field `Session-ID` is unlikely useful anymore
  - TODO, figure out what exactly `Resumption PSK` field means in openssl, where is it used in the entire TLS handshake ?
  - the field `TLS session ticket` contains octets which will be copied to `PskIdentity` field in the [pre-shared key extension](https://www.rfc-editor.org/rfc/rfc8446#section-4.2.11) of `ClientHello` message (for subsequent handshakes)
  - the name of the field `TLS session ticket` is confusing, as the *session tickets(RFC5077)* is obsolete.
- the field `PSK identity` is just a label, it is optional and can be empty.
- the field `TLS session ticket lifetime hint` means expiry time in seconds since the session was generated last time, in this example it is 420 seconds.
- another utility `s_client` can also dump the TLS session detail, without the option `-brief`
