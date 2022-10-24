extends KinematicBody2D

const MyProto = preload("res://goblin_proto_out.gd")
const MOTION_SPEED =80 # Pixels/second

var motion: Vector2

func _ready():
	motion = Vector2(0,0)

func _physics_process(_delta):
	motion = motion.normalized() * MOTION_SPEED
	move_and_slide(motion)
	
