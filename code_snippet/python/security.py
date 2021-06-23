
def birthday_sharing_prob(num_rounds=10000, maxval=365, num_ppl=23):
     collision_stat = {'yes':0, 'no': 0}
     for _ in range(num_rounds):
         days = [random.randint(0,maxval) for _ in range(num_ppl)]
         if len(days) == len(set(days)):
             collision_stat['no'] += 1
         else:
             collision_stat['yes'] += 1
     return collision_stat

