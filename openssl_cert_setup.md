### Quick Note

#### generate CA cert / private key
```
openssl genrsa -out ca_priv_key.pem  2048

openssl req -new -x509 -days 180 -key ca_priv_key.pem -keyform PEM -out ca_crt.pem -outform PEM -sha384

openssl x509 -text -noout -in ca_crt.pem -inform PEM
```

#### generate server cert, sign it using CA cert 

* x509v3 extension can be added by the option `extensions v3_req` and `-extfile <OPENSSL_INSTALL_PATH>/openssl.cnf`
* signature size is determined by RSA key kength (e.g. 2048 bits in the command below)
```
openssl genrsa -out server_priv_key.pem  2048

openssl req -new  -days 1 -key server_priv_key.pem -keyform PEM -out server_csr.pem -outform PEM  -sha384

openssl x509 -req -in server_csr.pem -extfile /usr/local/ssl/openssl.cnf -extensions v3_req  -CA ca_crt.pem -CAform PEM -CAkey ca_priv_key.pem -CAkeyform PEM -CAcreateserial -out server_crt.pem -outform PEM -days 1 -sha384
```



#### Read generated certificate
```
openssl x509 -text -noout -in server_crt.pem -inform PEM
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


