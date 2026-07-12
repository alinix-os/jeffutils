ROOT_DIR := $(realpath $(dir $(lastword $(MAKEFILE_LIST))))

MODE ?= unix
ARCH ?= x86
# ============================================================
# Rust toolchain
# ============================================================


REAL_USER := $(shell \
	if [ -n "$$SUDO_USER" ]; then \
		echo $$SUDO_USER; \
	else \
		echo $$USER; \
	fi \
)

REAL_HOME := $(shell eval echo "~$(REAL_USER)")

RUSTUP_HOME := $(REAL_HOME)/.rustup
CARGO_HOME := $(REAL_HOME)/.cargo

CARGO := $(CARGO_HOME)/bin/cargo

export RUSTUP_HOME
export CARGO_HOME

ifeq ($(wildcard $(CARGO)),)
CARGO := cargo
endif

SKIP ?=


# ============================================================
# Detect projects
# ============================================================

_RUST  := $(notdir $(patsubst %/Cargo.toml,%,$(wildcard $(ROOT_DIR)/*/Cargo.toml)))
_MAKE  := $(notdir $(patsubst %/Makefile,%,$(wildcard $(ROOT_DIR)/*/Makefile)))
_CMAKE := $(notdir $(patsubst %/CMakeLists.txt,%,$(wildcard $(ROOT_DIR)/*/CMakeLists.txt)))
_GO    := $(notdir $(patsubst %/go.mod,%,$(wildcard $(ROOT_DIR)/*/go.mod)))


_RUST  := $(filter-out $(SKIP),$(_RUST))
_MAKE  := $(filter-out $(SKIP),$(_MAKE))
_CMAKE := $(filter-out $(SKIP),$(_CMAKE))
_GO    := $(filter-out $(SKIP),$(_GO))


PROJECTS := $(sort $(_RUST) $(_MAKE) $(_CMAKE) $(_GO))


# ============================================================
# Targets
# ============================================================

TARGET_unix_x86    := x86_64-unknown-linux-gnu
TARGET_unix_arm    := aarch64-unknown-linux-gnu

TARGET_win_x86     := x86_64-pc-windows-msvc
TARGET_win_arm     := aarch64-pc-windows-msvc

TARGET_mac_x86     := x86_64-apple-darwin
TARGET_mac_arm     := aarch64-apple-darwin


# ============================================================
# Install
# ============================================================

INSTALL_unix    := /bin
INSTALL_mac     := /usr/local/bin
INSTALL_win     := C:/System32/JeffUtils


CARGO_TARGET := $(TARGET_$(MODE)_$(ARCH))

DESTDIR := $(INSTALL_$(MODE))

EXT := $(if $(filter win,$(MODE)),.exe,)

REL_DIR := target/$(if $(CARGO_TARGET),$(CARGO_TARGET)/)release



.PHONY: all build build-all install install-all clean info package-deb package-rpm package $(PROJECTS)



all: build


build: build-all



# ============================================================
# BUILD
# ============================================================

build-all:
	@echo ">> Building Rust projects (workspace)"
	@$(CARGO) build --release $(if $(CARGO_TARGET),--target $(CARGO_TARGET)) --workspace


	@for p in $(_MAKE); do \
		echo ">> Building $$p (Make)"; \
		$(MAKE) -C $(ROOT_DIR)/$$p || exit $$?; \
	done

	@for p in $(_CMAKE); do \
		echo ">> Building $$p (CMake)"; \
		mkdir -p $(ROOT_DIR)/$$p/build; \
		cd $(ROOT_DIR)/$$p/build && \
		cmake .. && \
		$(MAKE) || exit $$?; \
	done

	@for p in $(_GO); do \
		echo ">> Building $$p (Go)"; \
		cd $(ROOT_DIR)/$$p && \
		go build -o $(ROOT_DIR)/bin/$$p$(EXT) . || exit $$?; \
	done

	@echo "==> All projects built."



$(PROJECTS):
	@echo ">> Building $@"



# ============================================================
# INSTALL
# ============================================================

install install-all: build-all
	@for p in $(PROJECTS); do \
		echo ">> Installing $$p"; \
		sudo mkdir -p $(DESTDIR); \
		sudo cp $(ROOT_DIR)/$(REL_DIR)/$$p$(EXT) $(DESTDIR)/$$p$(EXT); \
	done

	@echo "==> All installed to $(DESTDIR)"



install/%:
	@echo ">> Installing $*"
	@sudo mkdir -p $(DESTDIR)
	@sudo cp \
	$(ROOT_DIR)/$(REL_DIR)/$*$(EXT) \
	$(DESTDIR)/$*$(EXT)




# ============================================================
# PACKAGE (.deb / .rpm)
# ============================================================
#
# Builds every utility (including jsh) plus the bundled JSH Mono font
# (jsh/assets/font) into a single jeffutils package via fpm. Requires fpm
# on PATH (gem install fpm), and dpkg-deb / rpmbuild for the respective
# format. Output lands in dist/.

package-deb: build-all
	@mkdir -p $(ROOT_DIR)/dist
	@REL_DIR=$(REL_DIR) $(ROOT_DIR)/packaging/build-pkg.sh deb

package-rpm: build-all
	@mkdir -p $(ROOT_DIR)/dist
	@REL_DIR=$(REL_DIR) $(ROOT_DIR)/packaging/build-pkg.sh rpm

package: package-deb package-rpm



# ============================================================
# CLEAN
# ============================================================

clean:

	@for p in $(_RUST); do \
		echo ">> Cleaning $$p"; \
		cd $(ROOT_DIR)/$$p && $(CARGO) clean; \
	done

	@for p in $(_MAKE); do \
		echo ">> Cleaning $$p"; \
		$(MAKE) -C $(ROOT_DIR)/$$p clean; \
	done

	@for p in $(_CMAKE); do \
		echo ">> Cleaning $$p"; \
		rm -rf $(ROOT_DIR)/$$p/build; \
	done

	@rm -rf $(ROOT_DIR)/dist

	@echo "==> All projects cleaned."



# ============================================================
# INFO
# ============================================================

info:

	@echo "JeffUtils Build System"
	@echo
	@echo "MODE = $(MODE)"
	@echo "ARCH = $(ARCH)"
	@echo "SKIP = $(SKIP)"
	@echo
	@echo "PROJECTS:"
	@for p in $(PROJECTS); do \
		echo " - $$p"; \
	done