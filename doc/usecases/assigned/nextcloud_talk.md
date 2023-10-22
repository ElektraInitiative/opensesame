# Talk With Nextcloud

## Summary

- **Scope:** Nextcloud Talk
- **Level:** User Goal
- **Actors** User, chat (Nextcloud Talk)
- **Brief:** Allow configuration of system via Nextcloud talk
- **Assignee:** Jannis
- **Status:** Assigned

## Scenarios

- **Precondition:** 
	- The device is on grid.
- **Main Success Scenario:** 
    - The user triggers opens door and switches light on/off via the chat.
    - The user asks about status (is door open?) and enabled/disabled modules in the chat. 
    - The user reconfigures the PIN codes in the chat.
- **Error scenario:**
	- The communication with Nextcloud doesn't work.
- **Postcondition:**
	- Bidirectional connection with Nextcloud talk chat.
- **Further Requirements:**
	- Module is extensible such that new chat commands can be introduced later.