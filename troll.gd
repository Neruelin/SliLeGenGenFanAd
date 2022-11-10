extends KinematicBody2D

const MyProto = preload("res://goblin_proto_out.gd")
const MOTION_SPEED = 80 # Pixels/second
const MOTION_THRESHOLD = 1 # Pixels/second

var motion: Vector2
var tar: Vector2

func _ready():
	motion = Vector2(0,0)
	tar = Vector2(0,0)
	
func new_loc(x, y):
	tar = Vector2(x, y)

func _physics_process(_delta):
	motion = tar - self.position
	if motion.length() > MOTION_THRESHOLD:
		#motion = motion.normalized() * MOTION_SPEED
		move_and_slide(motion * 5)
	
