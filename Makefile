# Thin wrapper: every target forwards to the justfile, which defines the recipes.
# Pass recipe arguments with ARGS, e.g. `make ctl ARGS="vm list"`.

JUST ?= just

.DEFAULT_GOAL := default

# Never try to remake this file through the catch-all rule.
Makefile: ;

%:
	@$(JUST) $@ $(ARGS)
