import random
import hashlib

def birthday_sharing_prob(num_rounds=10000, maxval=365, num_ppl=23):
     collision_stat = {'yes':0, 'no': 0}
     for _ in range(num_rounds):
         days = [random.randint(0,maxval) for _ in range(num_ppl)]
         if len(days) == len(set(days)):
             collision_stat['no'] += 1
         else:
             collision_stat['yes'] += 1
     return collision_stat


def md5_collision_example():
     collision_str1 = 'd131dd02c5e6eec4693d9a0698aff95c2fcab58712467eab4004583eb8fb7f8955ad340609f4b30283e488832571415a085125e8f7cdc99fd91dbdf280373c5bd8823e3156348f5bae6dacd436c919c6dd53e2b487da03fd02396306d248cda0e99f33420f577ee8ce54b67080a80d1ec69821bcb6a8839396f9652b6ff72a70'
     collision_str2 = 'd131dd02c5e6eec4693d9a0698aff95c2fcab50712467eab4004583eb8fb7f8955ad340609f4b30283e4888325f1415a085125e8f7cdc99fd91dbd7280373c5bd8823e3156348f5bae6dacd436c919c6dd53e23487da03fd02396306d248cda0e99f33420f577ee8ce54b67080280d1ec69821bcb6a8839396f965ab6ff72a70'
     md5objs = [hashlib.md5() for _ in range(2)]
     md5objs[0].update(bytearray.fromhex(collision_str1))
     md5objs[1].update(bytearray.fromhex(collision_str2))
     assert collision_str1 != collision_str2 , "the string contents should be different"
     assert md5objs[0].digest() == md5objs[1].digest() , "2 digests should be the same"
     
     
def sha1_collision_example():
     # the binary files (320 bytes per file) below come from :
     # https://privacylog.blogspot.com/2019/12/the-second-sha-collision.html
     filepaths = ['./sha1_collision_example_a_1.bin', './sha1_collision_example_a_2.bin']
     files = [open(filepaths[idx], 'rb') for idx in range(2)]
     sha1objs = [hashlib.sha1() for _ in range(2)]
     if files[0].read() != files[1].read():
          print("the string contents are different \n")
     
     for idx in range(2):
          files[idx].seek(0)
          sha1objs[idx].update( bytearray(files[idx].read()) )
          files[idx].close()
          
     if sha1objs[0].digest() == sha1objs[1].digest():
          print("but the digests from different file content are the same \n")

          
