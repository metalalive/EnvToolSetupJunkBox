#include <stdio.h>
#include <stdlib.h>

#include <openssl/opensslconf.h>
#include <openssl/bio.h>
#include <openssl/bn.h>
#include <openssl/rsa.h>
#include <openssl/evp.h>

#define DEFPRIMES 2
BIO *bio_err;

static int genrsa_cb(int p, int n, BN_GENCB *cb)
{
    return 1;
}


int main (){
    // deprecated since 3.0
    int ret = 1;
    int num = 256 << 3; // number of bits
    int primes = DEFPRIMES;
    const BIGNUM *e; // for returning public exponent from API
    char *hexe, *dece; // only for checking different forms of public exponent
    unsigned long f4 = RSA_F4; // default is 0x10001 , 2**n + 1 must be prime
    BIO *privkey_bio = NULL, *pubkey_bio = NULL;
    size_t  privkey_sz = 0 , pubkey_sz = 0;
    char   *privkey = NULL, *pubkey = NULL; // serializable C variable

    BN_GENCB *cb = BN_GENCB_new();
    BIGNUM *bn = BN_new();
    RSA *rsa = RSA_new();

    privkey_bio = BIO_new(BIO_s_mem());
    pubkey_bio = BIO_new(BIO_s_mem());

    BN_GENCB_set(cb, genrsa_cb, bio_err);
    if (!BN_set_word(bn, f4) || !RSA_generate_multi_prime_key(rsa, num, primes, bn, cb))
    {
        goto end;
    }
    RSA_get0_key(rsa, NULL, &e, NULL); // retrieve public exponent
    hexe = BN_bn2hex(e);
    dece = BN_bn2dec(e);
    if (hexe && dece) {
        printf("exponent e is %s (0x%s)\n", dece, hexe);
    }
    if (!PEM_write_bio_RSAPrivateKey(privkey_bio, rsa, NULL, NULL, 0, NULL, NULL))
        goto end;
    if (!PEM_write_bio_RSAPublicKey(pubkey_bio, rsa, NULL, NULL, 0, NULL, NULL))
        goto end;
    // print the retrieved private / public key
    privkey_sz = BIO_pending(privkey_bio); // pending ? get key size ?
    pubkey_sz  = BIO_pending(pubkey_bio);
    privkey = (char *)malloc(sizeof(char) * (privkey_sz + 1));
    pubkey  = (char *)malloc(sizeof(char) * (pubkey_sz + 1));
    BIO_read(privkey_bio, privkey, privkey_sz);
    BIO_read(pubkey_bio,  pubkey,  pubkey_sz);
    privkey[privkey_sz] = '\0';
    pubkey[pubkey_sz] = '\0';
    printf("retrieved private key: \n %s \n public key: \n %s \n",
            privkey, pubkey);
    ret = 0;
    printf("End of test, Generating RSA private key, %d bit long modulus (%d primes)\n",
                   num, primes);
end:
    OPENSSL_free(hexe);
    OPENSSL_free(dece);
    BN_GENCB_free(cb);
    BN_free(bn);
    RSA_free(rsa);
    BIO_free_all(privkey_bio);
    BIO_free_all(pubkey_bio );
    free(pubkey);
    free(privkey);
    return ret;
} // end of main()

// openssl genrsa -out rsa_private.pem 2048
// openssl rsa -in rsa_private.pem -outform PEM -pubout -out rsa_public.pem

// gcc -c -Wint-to-pointer-cast  -Wpointer-to-int-cast  -pthread  -Wall -fdata-sections -ffunction-sections -Wint-to-pointer-cast  -g -gdwarf-2 -Wa,-a,-ad -I/usr/local/include/ ./common/util/c/keygen.c -o tmp/keygen.o

// the order of object file and other options affects linking result .... wierd
// gcc ./tmp/keygen.o  -L/usr/local/lib  -lcrypto -lssl  -o ./tmp/keygen.out

