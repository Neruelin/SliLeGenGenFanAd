extends Node

var MESSAGES = {}
var MESSAGES_DECODE = []
var ws: WebSocketClient
var motion: Vector2
var config_data = {
	"server_addr": "127.0.0.1",
	"server_port": "9002"
}

class MessageSpec:
	var id: int
	var key: String
	var handler: FuncRef
	
	static func build(obj: Object, k: String, i: int, handler_name: String):
		var msgspc = MessageSpec.new()
		msgspc.key = k
		msgspc.id = i
		msgspc.handler = funcref(obj, handler_name)
		return msgspc

func load_config():
	# load settings from config file 	
	var config_file = File.new()
	assert(config_file.file_exists("res://config.json"), "Error: No config.json found.")
	config_file.open("res://config.json", File.READ)
	config_data = parse_json(config_file.get_as_text())

func init_ws():
	# create websocket client connection	
	ws = WebSocketClient.new()
	ws.connect("connection_established", self, "_established")
	ws.connect("connection_closed", self, "_closed")
	ws.connect("connection_error", self, "_closederror")
	ws.connect("data_received", self, "_data_received")
	ws.connect_to_url("ws://" + config_data.server_addr + ":" + config_data.server_port)
	print("Connecting to " + "ws://" + config_data.server_addr + ":" + config_data.server_port)

func init_MESSAGES():
	var i = 0
	MESSAGES_DECODE = []
	MESSAGES_DECODE.append(MessageSpec.build(self, "GET_ENTITIES_REQUEST", i, "handle_GET_ENTITIES"))
	i += 1
	MESSAGES_DECODE.append(MessageSpec.build(self, "NEW_ENTITY_REQUEST", i, "handle_NEW_ENTITY"))
	i += 1
	
	for msg in MESSAGES_DECODE:	
		MESSAGES[msg.key] = msg

# GetEntitiesRequest
func build_GetEntitiesRequest(x, y, h, w) -> PoolByteArray:
	var data = PoolByteArray()	
	data.append(MESSAGES["GET_ENTITIES_REQUEST"].id)
	data.append_array(pack_i32(x))
	data.append_array(pack_i32(y))
	data.append_array(pack_i32(h))
	data.append_array(pack_i32(w))
	return data
	
func handle_GET_ENTITIES(data):
	print("handle_GET_ENTITIES")	

# NewEntityRequest
func build_NewEntityRequest(x, y, id) -> PoolByteArray:
	var data = PoolByteArray()
	data.append(MESSAGES["NEW_ENTITY_REQUEST"].id)
	data.append_array(pack_i32(x))
	data.append_array(pack_i32(y))
	data.append(id)
	return data	
	
func handle_NEW_ENTITY(data):
	print("handle_NEW_ENTITY")
		
func route_data(data: PoolByteArray):
	assert(data.size() > 0, "Packet empty?")
	MESSAGES_DECODE[data[0]].handler.call_func(data)
	
func _established(_proto):
	print("connection established")
	ws.get_peer(1).put_packet(build_GetEntitiesRequest(123, 345, 789, 123))
	ws.get_peer(1).put_packet(build_NewEntityRequest(123, 345, 1))

func _data_received():
	route_data(ws.get_peer(1).get_packet())

func _closed(was_clean = false):
	print("Closed, clean: ", was_clean)
	set_process(false)

func _closederror(was_clean = false):
	print("Closederror, clean: ", was_clean)
	set_process(false)

func _ready():
	load_config()
	init_MESSAGES()
	init_ws()

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	ws.poll()	


### byte utils

func pack_i32(val) -> PoolByteArray:
	var value = PoolByteArray()
	for i in range(4):
		value.append(val >> ((3 - i) * 8) & 0b11111111)
	return value
	
func unpack_i32(buf) -> int:
	var stream = StreamPeerBuffer.new()
	stream.data_array = buf
	return stream.get_32()
