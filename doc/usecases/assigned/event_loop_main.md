# Modules and event loop refactor

## Summary

- **Scope:** Refactor main loops, thread spawing, signals into event loop with Tokio.
- **Level:** System Goal
- **Actors** System
- **Brief:** Replace messy main loops with async/await setup with Tokio. 
- **Assignee:** Jannis
- **Status:** Assigned

## Scenarios

- **Precondition:** 
	- Modules are clearly defined
- **Main Success Scenario:** 
    - Main is refactored to use async/await instead of thread spawning.
	- There is no more "normal mode" and "sensor mode", instead modules can be enabled or disabled.
- **Error scenario:**
	- No module works.
- **Postcondition:**
	- Ping always takes the same time.
	- Alarm mode can be ended.
	- Module errors are expected and handled.
- **Further Requirements:**
	- Better readabiliy of main.