#!/bin/sh

myplace=`dirname $0`

ssh-add
ansible-playbook -i $myplace/opensesame_dev_setup_inv.yaml $myplace/opensesame_dev_setup_pb.yaml --ask-become-pass
