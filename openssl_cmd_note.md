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
OpenSSL provides a test tool `s_client` acting as frontend client initiating TLS handshake. 
```shell
openssl s_client -CAfile /PATH/TO/CERTS/CA.crt   -tls1_3 -state  -sess_out \
    /PATH/TO/OUTPUT/SESSION-PARAM.pem   -connect localhost:1234
```
Note:
- session file can be saved to `/PATH/TO/OUTPUT/SESSION-PARAM.pem`, the content depends on your TLS version, e.g. in TLS 1.3, it includes pre-shared key ID / resumption key / ticket / cipher suites / other information.

To dump detail in the session file, use subcommand `sess_id`
```
openssl sess_id -in  /PATH/TO/OUTPUT/SESSION-PARAM.pem -text -noout
```
once session file is saved, you can use it for subsequent TLS handshakes to the same host/port by the expiry time
```
openssl s_client -noservername -sess_in  /PATH/TO/OUTPUT/SESSION-PARAM.pem  -sess_out  /PATH/TO/OUTPUT/SESSION-PARAM.pem   -tls1_3  -state  -connect localhost:1234
```
