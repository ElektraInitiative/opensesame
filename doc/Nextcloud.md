# Nextcloud Module
Implements two loops: one for sending (`message_sender_loop`) messages and status to Nextcloud, and the other for receiving (`command_loop`) messages/commands from Nextcloud.
Commands can be sent via Nextcloud chat by typing "\opensesame" to open the door, or other commands like "\ring_bell", "\fire_alarm", "\status", and "\switchlights true true".