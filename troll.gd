extends KinematicBody2D

const MyProto = preload("res://goblin_proto_out.gd")
const MOTION_SPEED =80 # Pixels/second

const MOVE_OP = 1

var ws: WebSocketClient
var motion: Vector2

var config_data = {
	"server_addr": "ws://127.0.0.1:9002",
	"server_port": "9002"
}

func _ready():
	# load settings from config file 
	var config_file = File.new()
	assert(config_file.file_exists("res://config.json"), "Error: No config.json found.")
	config_file.open("res://config.json", File.READ)
	var config_data = parse_json(config_file.get_as_text())
	
	# create websocket client connection
	ws = WebSocketClient.new()
	#warning-ignore:return_value_discarded
	ws.connect("connection_established", self, "_established")
	#warning-ignore:return_value_discarded
	ws.connect("connection_closed", self, "_closed")
	#warning-ignore:return_value_discarded
	ws.connect("connection_error", self, "_closederror")
	ws.connect("data_received", self, "_data_received")
	#warning-ignore:return_value_discarded

	ws.connect_to_url("ws://" + config_data.server_addr + ":" + config_data.server_port)
	print("trying to connect")

	motion = Vector2(0,0)


func _established(_proto):
	print("connection established")
	
	#warning-ignore:return_value_discarded
	# ws.get_peer(1).put_packet("testing".to_utf8())
	#var a = [1]
	#ws.get_peer(1).put_packet(a)
	#ws.get_peer(1).put_packet(a.to_bytes())
	#ws.get_peer(1).put_packet(a.to_bytes())

func route_data(data):
	var data_size = data.size()
	if (data_size > 0):
		if (data[0] == MOVE_OP && data_size == 3):
			motion.x = data[1] - 1
			motion.y = data[2] - 1
			print(motion)
	#print(data)

func _data_received():
	var dat = ws.get_peer(1).get_packet()
	route_data(dat)

func _closed(was_clean = false):
	print("Closed, clean: ", was_clean)
	set_process(false)

func _closederror(was_clean = false):
	print("Closederror, clean: ", was_clean)
	set_process(false)

func _physics_process(_delta):
	#var motion = Vector2()
	#motion.x = Input.get_action_strength("move_right") - Input.get_action_strength("move_left")
	#motion.y = Input.get_action_strength("move_down") - Input.get_action_strength("move_up")
	#motion.y /= 2
	motion = motion.normalized() * MOTION_SPEED
	#warning-ignore:return_value_discarded
	move_and_slide(motion)
	ws.poll()
	
