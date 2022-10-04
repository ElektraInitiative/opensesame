#!/bin/sh

myplace=`dirname $0`

ssh-add
ansible-playbook -i $myplace/inventory.yaml $myplace/playbook.yaml
