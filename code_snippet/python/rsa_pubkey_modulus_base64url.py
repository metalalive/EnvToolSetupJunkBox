import base64
import re

def pem_to_der(pem_data):
    """Convert PEM to DER."""
    pem_header = "-----BEGIN PUBLIC KEY-----"
    pem_footer = "-----END PUBLIC KEY-----"
    pem_data = pem_data.replace(pem_header, "").replace(pem_footer, "")
    return base64.b64decode(pem_data)

def extract_modulus_from_der(der_data):
    """Extract the modulus from DER-encoded RSA public key."""
    # Skip the sequence and length bytes (Assume standard structure for RSA public key)
    # This will need proper ASN.1 parsing for robustness.
    if der_data[:2] != b'\x30\x82':
        raise ValueError("Invalid DER sequence.")
    modulus_start = 30 # This is the starting byte of the modulus after headers
    modulus_len_bytes = 2
    modulus_content_start = modulus_start + modulus_len_bytes
    modulus_len = int.from_bytes(
        der_data[modulus_start:modulus_content_start],
        byteorder='big'
    )
    modulus = der_data[
        modulus_content_start: modulus_content_start + modulus_len
    ]
    #import pdb
    #pdb.set_trace()
    return modulus

def base64url_encode(data):
    """Base64url encode the given data."""
    data = data[1:]  # skip first zero byte for encoding JWK modulus
    return base64.urlsafe_b64encode(data).rstrip(b'=').decode('utf-8')

# Read the PEM file
pem_file_path = '/path/to/rsa256-pub-key.pem'
with open(pem_file_path, 'r') as pem_file:
    pem_data = pem_file.read()

# Convert PEM to DER
der_data = pem_to_der(pem_data)

# Extract the modulus from DER data
modulus = extract_modulus_from_der(der_data)

# Encode the modulus to base64url
modulus_base64url = base64url_encode(modulus)

print("Base64url encoded modulus for JWK :", modulus_base64url)
