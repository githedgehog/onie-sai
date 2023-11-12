# Copyright 2023 Hedgehog SONiC Foundation
# SPDX-License-Identifier: Apache-2.0
#
SHELL := bash
.SHELLFLAGS := -e -c
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules
MKFILE_DIR := $(shell echo $(dir $(abspath $(lastword $(MAKEFILE_LIST)))) | sed 'sA/$$AA')

CARGO_TARGET_DIR := $(MKFILE_DIR)/target

# NOTE: this will change once we add the operator
VERSION ?= $(shell git describe --tags --dirty --always)

GIT_COMMIT = $(shell git rev-parse HEAD)
GIT_DIRTY  = $(shell test -n "`git status --porcelain`" && echo "dirty" || echo "clean")
BUILD_DATE = $(shell date -u -Iseconds)

SRC_FILES := $(shell find $(MKFILE_DIR)/onie-sai -type f)
SRC_FILES += $(shell find $(MKFILE_DIR)/onie-sai-common -type f)
SRC_FILES += $(shell find $(MKFILE_DIR)/onie-sai-rpc -type f)
SRC_FILES += $(shell find $(MKFILE_DIR)/onie-saictl -type f)
SRC_FILES += $(shell find $(MKFILE_DIR)/onie-said -type f)
SRC_FILES += $(shell find $(MKFILE_DIR)/sai -type f)
SRC_FILES += $(shell find $(MKFILE_DIR)/sai-sys -type f)
SRC_FILES += $(shell find $(MKFILE_DIR)/xcvr -type f)
SRC_FILES += $(shell find $(MKFILE_DIR)/xcvr-sys -type f)
SRC_FILES += $(shell find $(MKFILE_DIR)/xcvr-dell-s5200 -type f)
SRC_FILES += $(shell find $(MKFILE_DIR)/xcvrctl -type f)

# This snippet has been sourced and adopted from the following file which is
# licensed under Apache-2.0:
# https://github.com/sonic-net/sonic-buildimage/blob/master/platform/broadcom/sai.mk
#
# --- BEGIN SNIPPET ---
LIBSAIBCM_XGS_VERSION = 8.4.0.2
LIBSAIBCM_XGS_BRANCH_NAME = SAI_8.4.0_GA
LIBSAIBCM_XGS_URL_PREFIX = "https://sonicstorage.blob.core.windows.net/public/sai/sai-broadcom/$(LIBSAIBCM_XGS_BRANCH_NAME)/$(LIBSAIBCM_XGS_VERSION)/xgs"

BRCM_XGS_SAI_DEB = libsaibcm_$(LIBSAIBCM_XGS_VERSION)_amd64.deb
BRCM_XGS_SAI_DEV_DEB = libsaibcm-dev_$(LIBSAIBCM_XGS_VERSION)_amd64.deb
# --- END SNIPPET ---
DOWNLOAD_DIR := $(MKFILE_DIR)/dl
SAIBCM_DIR := $(MKFILE_DIR)/dl/saibcm

.PHONY: help
help: ## Display this help.
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_0-9-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

all: init build ## Runs 'init' and 'build' targets

init: download_libsai extract_libsai ## Ensures all dependencies for the project are in place

build: onie-sai  ## Builds the project

all-clean: cargo-clean lib-clean download-clean ## Cleans the whole project directory, and deletes downloaded dependencies

clean: cargo-clean ## Cleans the project directory

download_libsai: $(DOWNLOAD_DIR)/$(BRCM_XGS_SAI_DEB) $(DOWNLOAD_DIR)/$(BRCM_XGS_SAI_DEV_DEB)

$(DOWNLOAD_DIR)/$(BRCM_XGS_SAI_DEB) $(DOWNLOAD_DIR)/$(BRCM_XGS_SAI_DEV_DEB) &:
	wget -O $(DOWNLOAD_DIR)/$(BRCM_XGS_SAI_DEB) $(LIBSAIBCM_XGS_URL_PREFIX)/$(BRCM_XGS_SAI_DEB)
	wget -O $(DOWNLOAD_DIR)/$(BRCM_XGS_SAI_DEV_DEB) $(LIBSAIBCM_XGS_URL_PREFIX)/$(BRCM_XGS_SAI_DEV_DEB)

extract_libsai: $(MKFILE_DIR)/lib/libsai.so $(MKFILE_DIR)/lib/libsai.so.1 $(MKFILE_DIR)/lib/libsai.so.1.0

$(MKFILE_DIR)/lib/libsai.so $(MKFILE_DIR)/lib/libsai.so.1 $(MKFILE_DIR)/lib/libsai.so.1.0 &:
	mkdir -p $(SAIBCM_DIR) && cd $(SAIBCM_DIR) && \
	ar x $(DOWNLOAD_DIR)/$(BRCM_XGS_SAI_DEB) && \
	tar -xf data.tar.xz && \
	rm -f data.tar.xz control.tar.xz debian-binary && \
	ar x $(DOWNLOAD_DIR)/$(BRCM_XGS_SAI_DEV_DEB) && \
	tar -xf data.tar.xz && \
	rm -f data.tar.xz control.tar.xz debian-binary && \
	cp -Pav $(SAIBCM_DIR)/usr/lib/* $(MKFILE_DIR)/lib/ && \
	chmod -v +x $(MKFILE_DIR)/lib/*.so*

onie-sai: $(CARGO_TARGET_DIR)/release/onie-sai ## Builds 'onie-sai' for the release target

$(CARGO_TARGET_DIR)/release/onie-sai: $(SRC_FILES) extract_libsai
	LD_LIBRARY_PATH="$(MKFILE_DIR)/lib:$$LD_LIBRARY_PATH" \
		cargo build --release --workspace

.PHONY: cargo-clean
cargo-clean: ## Cleans the whole target/ directory
	cargo clean || true

.PHONY: lib-clean
lib-clean: ## Cleans the lib/ directory
	rm -rvf $(MKFILE_DIR)/lib/*.so* || true

.PHONY: download-clean
download-clean: ## Cleans the dl/ directory
	rm -rvf $(DOWNLOAD_DIR)/* || true
