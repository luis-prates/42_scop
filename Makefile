NAME = scop_42
CARGO = cargo
RM = rm -rf
TARGET_DIR = target
DEBUG_BIN = $(TARGET_DIR)/debug/$(NAME)
RELEASE_BIN = $(TARGET_DIR)/release/$(NAME)

# Default resources for testing
DEFAULT_MODEL = resources/models/42.obj
DEFAULT_TEXTURE = resources/textures/brickwall.bmp

# Colors for output
GREEN = \033[0;32m
YELLOW = \033[0;33m
RED = \033[0;31m
NC = \033[0m

all: $(NAME)

$(NAME):
	@echo "$(GREEN)Building $(NAME) in debug mode...$(NC)"
	@$(CARGO) build
	@echo "$(GREEN)Build complete: $(DEBUG_BIN)$(NC)"

release:
	@echo "$(GREEN)Building $(NAME) in release mode...$(NC)"
	@$(CARGO) build --release
	@echo "$(GREEN)Release build complete: $(RELEASE_BIN)$(NC)"

# Run in debug mode (requires MODEL and TEXTURE arguments)
run:
	@if [ -z "$(MODEL)" ] || [ -z "$(TEXTURE)" ]; then \
		echo "$(YELLOW)Usage: make run MODEL=<model_path> TEXTURE=<texture_path>$(NC)"; \
		echo "$(YELLOW)Example: make run MODEL=resources/models/42.obj TEXTURE=resources/textures/brickwall.bmp$(NC)"; \
		echo ""; \
		echo "$(YELLOW)Or use default resources:$(NC)"; \
		echo "$(YELLOW)  make run-default$(NC)"; \
	else \
		echo "$(GREEN)Running $(NAME) with MODEL=$(MODEL) TEXTURE=$(TEXTURE)$(NC)"; \
		$(CARGO) run -- $(MODEL) $(TEXTURE); \
	fi

# Run in release mode (requires MODEL and TEXTURE arguments)
run-release:
	@if [ -z "$(MODEL)" ] || [ -z "$(TEXTURE)" ]; then \
		echo "$(YELLOW)Usage: make run-release MODEL=<model_path> TEXTURE=<texture_path>$(NC)"; \
		echo "$(YELLOW)Example: make run-release MODEL=resources/models/42.obj TEXTURE=resources/textures/brickwall.bmp$(NC)"; \
		echo ""; \
		echo "$(YELLOW)Or use default resources:$(NC)"; \
		echo "$(YELLOW)  make run-release-default$(NC)"; \
	else \
		echo "$(GREEN)Running $(NAME) (release) with MODEL=$(MODEL) TEXTURE=$(TEXTURE)$(NC)"; \
		$(CARGO) run --release -- $(MODEL) $(TEXTURE); \
	fi

# Run with default resources
run-default:
	@echo "$(GREEN)Running $(NAME) with default resources...$(NC)"
	@if [ -f "$(DEFAULT_MODEL)" ] && [ -f "$(DEFAULT_TEXTURE)" ]; then \
		$(CARGO) run -- $(DEFAULT_MODEL) $(DEFAULT_TEXTURE); \
	else \
		echo "$(RED)Default resources not found!$(NC)"; \
		echo "$(YELLOW)Please ensure $(DEFAULT_MODEL) and $(DEFAULT_TEXTURE) exist$(NC)"; \
	fi

run-release-default:
	@echo "$(GREEN)Running $(NAME) (release) with default resources...$(NC)"
	@if [ -f "$(DEFAULT_MODEL)" ] && [ -f "$(DEFAULT_TEXTURE)" ]; then \
		$(CARGO) run --release -- $(DEFAULT_MODEL) $(DEFAULT_TEXTURE); \
	else \
		echo "$(RED)Default resources not found!$(NC)"; \
		echo "$(YELLOW)Please ensure $(DEFAULT_MODEL) and $(DEFAULT_TEXTURE) exist$(NC)"; \
	fi

# Check code without building
check:
	@echo "$(GREEN)Checking code...$(NC)"
	@$(CARGO) check

# Run tests
test:
	@echo "$(GREEN)Running tests...$(NC)"
	@$(CARGO) test

# Clean build artifacts (keeps dependencies)
clean:
	@echo "$(YELLOW)Cleaning build artifacts...$(NC)"
	@$(CARGO) clean
	@echo "$(GREEN)Clean complete$(NC)"

# Full clean (same as clean for Cargo projects)
fclean: clean
	@echo "$(GREEN)Full clean complete$(NC)"

# Rebuild from scratch
re: fclean all

# Format code
fmt:
	@echo "$(GREEN)Formatting code...$(NC)"
	@$(CARGO) fmt

# Documentation lint
doclint:
	@echo "$(GREEN)Linting project docs...$(NC)"
	@python3 scripts/doc_lint.py

# Agent guardrail checks
agent-check:
	@echo "$(GREEN)Running agent check script...$(NC)"
	@bash scripts/agent_check.sh

# One-shot verification
verify:
	@echo "$(GREEN)Running one-shot verification...$(NC)"
	@$(MAKE) doclint
	@bash scripts/agent_check.sh
	@echo "$(GREEN)Verification complete$(NC)"

# Show help
help:
	@echo "$(GREEN)scop_42 Makefile$(NC)"
	@echo ""
	@echo "$(YELLOW)Build targets:$(NC)"
	@echo "  make            - Build debug version"
	@echo "  make release    - Build release version (optimized)"
	@echo "  make check      - Check code without building"
	@echo ""
	@echo "$(YELLOW)Run targets:$(NC)"
	@echo "  make run MODEL=<path> TEXTURE=<path>  - Run debug version"
	@echo "  make run-release MODEL=<path> TEXTURE=<path>  - Run release version"
	@echo "  make run-default          - Run with default resources (debug)"
	@echo "  make run-release-default  - Run with default resources (release)"
	@echo ""
	@echo "$(YELLOW)Clean targets:$(NC)"
	@echo "  make clean      - Remove build artifacts"
	@echo "  make fclean     - Full clean"
	@echo "  make re         - Rebuild from scratch"
	@echo ""
	@echo "$(YELLOW)Development targets:$(NC)"
	@echo "  make test       - Run tests"
	@echo "  make fmt        - Format code"
	@echo "  make doclint    - Lint required docs and reviews banner rules"
	@echo "  make verify     - Run doclint + fmt/check (+clippy when available)"
	@echo "  make help       - Show this help"

.PHONY: all release run run-release run-default run-release-default \
        check test clean fclean re fmt clippy doclint agent-check verify help
