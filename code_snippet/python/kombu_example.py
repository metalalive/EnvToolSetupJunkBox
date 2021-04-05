import sys
import json
import pdb
import kombu

_data = {'myqueue': None, 'myexchange':None}

def _init(queue_name='usermgt_rpc3_rx' , exchange_name='usermgt_rpc3_rx', exchange_type='topic'):
    # note: queue name and exchange name can be different
    _data['myexchange'] = kombu.Exchange(name=exchange_name, type=exchange_type, )
    # each queue could have multiple bindings, each of which is bound with different
    # routing key, so routing key doesn't have to be configured at here
    _data['myqueue']  = kombu.Queue(name=queue_name, exchange=_data['myexchange'])

def get_message(body, message):
    print("receive message: %s" % body)
    print("what is inside message object : %s" % message)
    message.ack()

def subscribe(auth_url, routing_key):
    _data['myqueue'].routing_key = routing_key
    with kombu.Connection(auth_url) as conn:
        with kombu.Consumer(channel=conn.default_channel, queues=[_data['myqueue']],
                callbacks=[get_message]) as consumer :
            # if adding queue afterward, the code still work , but raise exception
            # at the end of the context meneger. See [1] for stack trace
            #consumer.add_queue(_data['myqueue'])
            # in this example, the callback function will acknowledge the received message
            consumer.consume(no_ack=True)
    print('--- end of consumer --- ')


def publish(auth_url, msg_body, routing_key, headers=None):
    if isinstance(headers, str):
        headers = json.loads(headers)
    if isinstance(msg_body, str):
        msg_body = json.loads(msg_body)
    with kombu.Connection(auth_url) as conn:
        producer = kombu.Producer(channel=conn.default_channel)
        pub_result = producer.publish(body=msg_body, routing_key=routing_key,
                exchange=_data['myexchange'], headers=headers )
        if not pub_result.ready:
            print('[WARNING] the published message may not be ready in the broker')
    # conn.release() # manually close the connection if it's not in context manager
    print('--- end of publish --- ')


def _publish_to_celery_consumer(auth_url, routing_key, headers=None, msg_payld=None):
    # essential headers sent to python Celery consumers (workers):
    # task: full hierarchical path of the Celery task function (as a consumer)
    # id: Celery defaults to UUID (v4 ?). TODO figure out whether other format of ID could work
    default_headers = {'shadow':None, 'task': 'user_management.async_tasks.remote_get_profile',
            'id':'1a862d39-7c1f-4645-8928-cbf3e9478c9f', 'content_type':'application/json',
            }
    # message payload to Celery consumer has to be serializable array of 3 items:
    # array[0] : ??
    # array[1] : args of the consumer task function, in general it should be a dictionary object
    # array[2] : metadata referenced by kombu library
    default_msg_payld = [[], {'account_id': 10, 'field_names':['id', 'first_name', 'last_name']},
            {'callbacks': None, 'errbacks': None, 'chain': None, 'chord': None}]
    headers = headers or default_headers
    msg_payld = msg_payld or default_msg_payld
    publish(auth_url=auth_url, msg_body=msg_payld, routing_key=routing_key, headers=headers)

# {'lang': 'py', 'task': 'user_management.async_tasks.update_roles_on_accounts', 'id': '1a862d39-7c1f-4645-8928-cbf3e9478c9f', 'shadow': None, 'eta': None, 'expires': None, 'group': None, 'group_index': None, 'retries': 0, 'timelimit': [None, None], 'root_id': '1a862d39-7c1f-4645-8928-cbf3e9478c9f', 'parent_id': None, 'argsrepr': '()', 'kwargsrepr': "{'affected_groups': [118]}", 'origin': 'gen6703@localhost'}

# [[], {'affected_groups': [118]}, {'callbacks': None, 'errbacks': None, 'chain': None, 'chord': None}]




def declare(auth_url, routing_key):
    _data['myqueue'].durable = True
    _data['myqueue'].max_length_bytes = 4096
    with kombu.Connection(auth_url) as conn:
        bound_exchange = _data['myexchange'](conn.default_channel)
        bound_exchange.declare()
        bound_myq = _data['myqueue'](conn.default_channel)
        bound_myq.declare()
        bind_q_ex = bound_myq.bind_to(exchange=bound_exchange.name , routing_key=routing_key)

def delete(auth_url, routing_key):
    with kombu.Connection(auth_url) as conn:
        bound_exchange = _data['myexchange'](conn.default_channel)
        bound_myq = _data['myqueue'](conn.default_channel)
        bind_q_ex = bound_myq.unbind_from(exchange=bound_exchange, routing_key=routing_key)
        bound_exchange.delete()
        bound_myq.delete()


def main():
    fn = None
    init_kwargs = {}
    fn_kwargs  = {}

    if sys.argv[1] == 'subscribe':
        fn = subscribe
        fn_kwargs.update({'auth_url':sys.argv[2], 'routing_key':sys.argv[3]})
        init_kwargs = {'queue_name': sys.argv[4], 'exchange_name':sys.argv[5],
                'exchange_type':sys.argv[6]}
    elif sys.argv[1] == 'publish':
        fn = publish
        fn_kwargs.update({'auth_url':sys.argv[2], 'routing_key':sys.argv[3],
            'msg_body':sys.argv[4]})
        init_kwargs = {'queue_name': sys.argv[5], 'exchange_name':sys.argv[6],
                'exchange_type':sys.argv[7]}
    elif sys.argv[1] == 'publish_to_celery':
        fn = _publish_to_celery_consumer
        fn_kwargs.update({'auth_url':sys.argv[2], 'routing_key':sys.argv[3],
            'msg_payld':sys.argv[4], 'headers':sys.argv[5] })
        init_kwargs = {'queue_name': sys.argv[6], 'exchange_name':sys.argv[7],
                'exchange_type':sys.argv[8]}
    elif sys.argv[1] == 'declare':
        fn = declare
        fn_kwargs.update({'auth_url':sys.argv[2], 'routing_key':sys.argv[3] })
        init_kwargs = {'queue_name': sys.argv[4], 'exchange_name':sys.argv[5],
                'exchange_type':sys.argv[6]}
    elif sys.argv[1] == 'delete':
        fn = delete
        fn_kwargs.update({'auth_url':sys.argv[2], 'routing_key':sys.argv[3] })
        init_kwargs = {'queue_name': sys.argv[4], 'exchange_name':sys.argv[5],
                'exchange_type':sys.argv[6]}

    if fn:
        _init(**init_kwargs)
        fn(**fn_kwargs)


if __name__ == "__main__":
    main()

"""
[1]
  File "site-packages/kombu/connection.py", line 821, in __exit__
    self.release()
  File "site-packages/kombu/connection.py", line 375, in release
    self._close()
  File "site-packages/kombu/connection.py", line 341, in _close
    self._do_close_self()
  File "site-packages/kombu/connection.py", line 331, in _do_close_self
    self.maybe_close_channel(self._default_channel)
  File "site-packages/kombu/connection.py", line 323, in maybe_close_channel
    channel.close()
  File "site-packages/amqp/channel.py", line 219, in close
    return self.send_method(
  File "site-packages/amqp/abstract_channel.py", line 66, in send_method
    return self.wait(wait, returns_tuple=returns_tuple)
  File "site-packages/amqp/abstract_channel.py", line 86, in wait
    self.connection.drain_events(timeout=timeout)
  File "site-packages/amqp/connection.py", line 514, in drain_events
    while not self.blocking_read(timeout):
  File "site-packages/amqp/connection.py", line 520, in blocking_read
    return self.on_inbound_frame(frame)
  File "site-packages/amqp/method_framing.py", line 53, in on_frame
    callback(channel, method_sig, buf, None)
  File "site-packages/amqp/connection.py", line 526, in on_inbound_method
    return self.channels[channel_id].dispatch_method(
  File "site-packages/amqp/abstract_channel.py", line 143, in dispatch_method
    listener(*args)
  File "site-packages/amqp/channel.py", line 276, in _on_close
    self._do_revive()
  File "site-packages/amqp/channel.py", line 161, in _do_revive
    self.open()
  File "site-packages/amqp/channel.py", line 432, in open
    return self.send_method(
  File "site-packages/amqp/abstract_channel.py", line 66, in send_method
    return self.wait(wait, returns_tuple=returns_tuple)
  File "site-packages/amqp/abstract_channel.py", line 86, in wait
    self.connection.drain_events(timeout=timeout)
AttributeError: 'NoneType' object has no attribute 'drain_events'

[2] usage
python3.9 kombu_example.py  declare  amqp://USERNAME:PASSWORD@localhost:5672  UserMgtRPC.getprofile.#  usermgt_rpc2_rx  usermgt_rpc2_rx  topic
python3.9 kombu_example.py  delete   amqp://USERNAME:PASSWORD@localhost:5672  UserMgtRPC.getprofile.#  usermgt_rpc2_rx  usermgt_rpc2_rx  topic
python3.9 kombu_example.py  publish  amqp://USERNAME:PASSWORD@localhost:5672  UserMgtRPC.getprofile.ANY_CHAR  "{\"modern way to host game backend\": \"orchestra  statistics\"}"      usermgt_rpc2_rx  usermgt_rpc2_rx  topic

python3.9 kombu_example.py  publish_to_celery  amqp://USERNAME:PASSWORD@localhost:5672  UserMgtRPC.getprofile.ANY_CHAR            "[[], {\"account_id\": 10, \"field_names\":[\"id\", \"first_name\", \"last_name\"]}, {\"callbacks\": null, \"errbacks\": null, \"chain\": null, \"chord\": null}]"          "{\"shadow\":null, \"task\": \"user_management.async_tasks.remote_get_profile\", \"id\":\"1a862d39-7c1f-4645-8928-cbf3e9478c9f\", \"content_type\":\"application/json\"}"    usermgt_rpc2_rx  usermgt_rpc2_rx  topic

python3.9 kombu_example.py  subscribe amqp://USERNAME:PASSWORD@localhost:5672  UserMgtRPC.getprofile.#    usermgt_rpc2_rx  usermgt_rpc2_rx  topic

"""

