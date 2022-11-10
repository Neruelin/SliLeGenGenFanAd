extends Node

var troll = preload("res://troll.tscn")

var MESSAGES = {}
var MESSAGES_DECODE = []
var entitiesList = {}
var activeEntity = -1
var ws: WebSocketClient
var ws_status = 0;
# 0 never connected, 1 connected, -1 reconnecting
var config_data = {
	"server_addr": "127.0.0.1",
	"server_port": "9002"
}

func handle_entity_update(update):
	for up in update:
		if entitiesList.has(up[0]):
			entitiesList[up[0]].new_loc(up[1], up[2])
			#entitiesList[up[0]].position.x = up[1]
			#entitiesList[up[0]].position.y = up[2]
		else:
			var t = troll.instance()
			t.position.x = up[1]
			t.position.y = up[2]
			t.new_loc(up[1], up[2])
			entitiesList[up[0]] = t
			add_child(t)

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

class MessageDecoder:
	var data_stream: StreamPeerBuffer
	var cursor: int
	
	func _init(data):
		data_stream = StreamPeerBuffer.new()
		data_stream.data_array = data
		cursor = 0
		
	func get_u8():
		data_stream.seek(cursor)
		var val = data_stream.get_u8()
		cursor += 1
		return val
		
	func get_u32():
		data_stream.seek(cursor)
		var val = data_stream.get_u32()
		cursor += 4
		return val
		
func load_config():
	# load settings from config file 	
	var config_file = File.new()
	assert(config_file.file_exists("res://config.json"), "Error: No config.json found.")
	config_file.open("res://config.json", File.READ)
	config_data = parse_json(config_file.get_as_text())

func init_ws():
	# create websocket client connection	
	ws_status = 0
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
	MESSAGES_DECODE.append(MessageSpec.build(self, "CLEAR_ENTITIES_REQUEST", i, "handle_CLEAR_ENTITIES"))
	i += 1
	MESSAGES_DECODE.append(MessageSpec.build(self, "MOVE_ENTITY_REQUEST", i, "handle_MOVE_ENTITY"))
	i += 1
	MESSAGES_DECODE.append(MessageSpec.build(self, "CREATE_CONTROL_ENTITY_REQUEST", i, "handle_CREATE_CONTROL_ENTITY"))
	i += 1
	MESSAGES_DECODE.append(MessageSpec.build(self, "DISCONNECT_REQUEST", i, "handle_DISCONNECT"))
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
	var strem = MessageDecoder.new(data)
	var op = strem.get_u8()
	var amt = strem.get_u8()
	var updates = []
	for x in amt:
		var temp = [strem.get_u32(), strem.get_u32(), strem.get_u32()]
		updates.push_back(temp)
	if amt > 0:
		handle_entity_update(updates)

# NewEntityRequest
func build_NewEntityRequest(x, y, id) -> PoolByteArray:
	var data = PoolByteArray()
	data.append(MESSAGES["NEW_ENTITY_REQUEST"].id)
	data.append_array(pack_i32(x))
	data.append_array(pack_i32(y))
	data.append_array(pack_i32(id))
	return data	
func handle_NEW_ENTITY(data):
	print("handle_NEW_ENTITY")
	
# ClearEntitiesRequest
func build_ClearEntitiesRequest() -> PoolByteArray:
	var data = PoolByteArray()
	data.append(MESSAGES["CLEAR_ENTITIES_REQUEST"].id)
	return data
func handle_CLEAR_ENTITIES(data):
	print("handle_CLEAR_ENTITIES")
	
# MoveEntityRequest
func build_MoveEntityRequest(x, y, id) -> PoolByteArray:
	var data = PoolByteArray()
	data.append(MESSAGES["MOVE_ENTITY_REQUEST"].id)
	data.append_array(pack_i32(x))
	data.append_array(pack_i32(y))
	data.append_array(pack_i32(id))
	return data
func handle_MOVE_ENTITY(data):
	print("handle_MOVE_ENTITY")
	
# CreateControlEntityRequest
func build_CreateControlEntityRequest() -> PoolByteArray:
	var data = PoolByteArray()
	data.append(MESSAGES["CREATE_CONTROL_ENTITY_REQUEST"].id)
	return data
func handle_CREATE_CONTROL_ENTITY(data):
	var strem = MessageDecoder.new(data)
	var op = strem.get_u8();
	var update = [strem.get_u32(), strem.get_u32(), strem.get_u32()]
	print(update)
	handle_entity_update([update])
	activeEntity = update[0];
	print("handle_CREATE_CONTROL_ENTITY")
	
# DisconnectRequest
func build_DisconnectRequest() -> PoolByteArray:
	var data = PoolByteArray()
	data.append(MESSAGES["DISCONNECT_REQUEST"].id)
	data.append_array(pack_i32(activeEntity))
	return data
func handle_DISCONNECT(data):
	print("handle_DISCONNECT")
		
func route_data(data: PoolByteArray):
	assert(data.size() > 0, "Packet empty?")
	MESSAGES_DECODE[data[0]].handler.call_func(data)
	
func _established(_proto):
	print("connection established")
	if ws_status == 0:
		#ws.get_peer(1).put_packet(build_NewEntityRequest(500, 500, 12))
		ws.get_peer(1).put_packet(build_CreateControlEntityRequest())
	ws_status = 1

func _data_received():
	route_data(ws.get_peer(1).get_packet())

func _closed(was_clean = false):
	ws_status = -1
	print("Closed, clean: ", was_clean)
	set_process(false)

func _closederror(was_clean = false):
	ws_status = -1	
	print("Closederror, clean: ", was_clean)
	set_process(false)

func _ready():
	load_config()
	init_MESSAGES()
	init_ws()

func _notification(what):
	if what == MainLoop.NOTIFICATION_WM_QUIT_REQUEST:
		if activeEntity != -1:
			ws.get_peer(1).put_packet(build_DisconnectRequest())
		ws.disconnect_from_host()
		get_tree().quit() # default behavior

var acc = 0
var acc_threshold = 0.1
var ws_reconnect_acc = 0
var ws_reconnect_threshold = 10
var move_delay_acc = 0
var move_delay_threshold = 0.05

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	ws.poll()	
	if ws_status < 1:
		ws_reconnect_acc += delta
		if ws_reconnect_acc > ws_reconnect_threshold:
			ws_reconnect_acc = 0
			init_ws()
	else:
		acc += delta
		move_delay_acc += delta
		if acc > acc_threshold:
			ws.get_peer(1).put_packet(build_GetEntitiesRequest(0, 0, 1000, 1000))
			acc = 0
		if move_delay_acc > move_delay_threshold:
			move_delay_acc = 0
			if activeEntity != -1 && entitiesList.has(activeEntity):
				var x = 0
				var y = 0
				if Input.is_action_pressed("move_down"):
					y += 1
				if Input.is_action_pressed("move_up"):
					y -= 1
				if Input.is_action_pressed("move_right"):
					x += 1
				if Input.is_action_pressed("move_left"):
					x -= 1
				ws.get_peer(1).put_packet(build_MoveEntityRequest(x, y, activeEntity))


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

### message types

var OUTGOING_MESSAGES = []
var INCOMING_MESSAGES = []
