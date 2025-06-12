.PHONY: all watch build help

# Default target
all: help

# Check for npm installation
NPM_EXISTS := $(shell command -v npm 2> /dev/null)

# Install dependencies if node_modules doesn't exist
node_modules:
	@if [ ! -d "node_modules" ]; then \
		echo "Installing dependencies..."; \
		npm install; \
	fi

# Watch mode for development
watch: node_modules
ifndef NPM_EXISTS
	$(error Error: npm is not installed. Please install Node.js and npm first.)
endif
	@echo "Starting Tailwind CSS compiler in watch mode..."
	@npm run watch:css

# Build for production
build: node_modules
ifndef NPM_EXISTS
	$(error Error: npm is not installed. Please install Node.js and npm first.)
endif
	@echo "Building Tailwind CSS for production..."
	@npm run build:css

# Show help message
help:
	@echo "Usage: make [target]"
	@echo "Targets:"
	@echo "  watch    - Run in watch mode for development"
	@echo "  build    - Build for production (minified)"
	@echo "  help     - Show this help message"
