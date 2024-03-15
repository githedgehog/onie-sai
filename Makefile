# Copyright 2023 Hedgehog SONiC Foundation
# SPDX-License-Identifier: Apache-2.0
#
SHELL := bash
.SHELLFLAGS := -e -c
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules
MKFILE_DIR := $(shell echo $(dir $(abspath $(lastword $(MAKEFILE_LIST)))) | sed 'sA/$$AA')

# NOTE: this will change once we add the operator
VERSION ?= $(shell git describe --tags --dirty --always)
CARGO_TARGET_DIR := $(MKFILE_DIR)/target
CARGO_TARGETS := $(CARGO_TARGET_DIR)/release/onie-sai $(CARGO_TARGET_DIR)/release/onie-lldp $(CARGO_TARGET_DIR)/release/libxcvr_dell_s5200.so

# helping Makefile to track if cargo build needs to run
SRC_FILES := $(shell find $(MKFILE_DIR)/onie-lldp -type f)
SRC_FILES := $(shell find $(MKFILE_DIR)/onie-sai -type f)
SRC_FILES += $(shell find $(MKFILE_DIR)/onie-sai-common -type f)
SRC_FILES += $(shell find $(MKFILE_DIR)/onie-sai-rpc -type f)
SRC_FILES += $(shell find $(MKFILE_DIR)/onie-saictl -type f)
SRC_FILES += $(shell find $(MKFILE_DIR)/onie-said -type f)
SRC_FILES += $(shell find $(MKFILE_DIR)/sai -type f)
SRC_FILES += $(shell find $(MKFILE_DIR)/sai-sys -type f)
SRC_FILES += $(shell find $(MKFILE_DIR)/xcvr -type f)
SRC_FILES += $(shell find $(MKFILE_DIR)/xcvr-sys -type f)
SRC_FILES += $(shell find $(MKFILE_DIR)/xcvr-cel-seastone2 -type f)
SRC_FILES += $(shell find $(MKFILE_DIR)/xcvr-cel-silverstone -type f)
SRC_FILES += $(shell find $(MKFILE_DIR)/xcvr-dell-s5200 -type f)
SRC_FILES += $(shell find $(MKFILE_DIR)/xcvr-pddf -type f)
SRC_FILES += $(shell find $(MKFILE_DIR)/xcvrctl -type f)

# all things for packaging
PACKAGE_ARTIFACTS_DIR := $(MKFILE_DIR)/artifacts
ARCH := $(shell uname -m)
PACKAGE_CORE_DIR := onie-sai-$(VERSION)-linux-$(ARCH)
PACKAGE_CORE_FILE := $(PACKAGE_ARTIFACTS_DIR)/$(PACKAGE_CORE_DIR).tar.gz
PACKAGE_LLDP_DIR := onie-lldp-$(VERSION)-linux-$(ARCH)
PACKAGE_LLDP_FILE := $(PACKAGE_ARTIFACTS_DIR)/$(PACKAGE_LLDP_DIR).tar.gz
PACKAGE_XCVR_DELL_S5200_DIR := onie-sai-xcvr-dell-s5200-$(VERSION)-linux-$(ARCH)
PACKAGE_XCVR_DELL_S5200_FILE := $(PACKAGE_ARTIFACTS_DIR)/$(PACKAGE_XCVR_DELL_S5200_DIR).tar.gz
PACKAGE_XCVR_PDDF_DIR := onie-sai-xcvr-pddf-$(VERSION)-linux-$(ARCH)
PACKAGE_XCVR_PDDF_FILE := $(PACKAGE_ARTIFACTS_DIR)/$(PACKAGE_XCVR_PDDF_DIR).tar.gz
PACKAGE_XCVR_CEL_SEASTONE2_DIR := onie-sai-xcvr-cel-seastone2-$(VERSION)-linux-$(ARCH)
PACKAGE_XCVR_CEL_SEASTONE2_FILE := $(PACKAGE_ARTIFACTS_DIR)/$(PACKAGE_XCVR_CEL_SEASTONE2_DIR).tar.gz
PACKAGE_XCVR_CEL_SILVERSTONE_DIR := onie-sai-xcvr-cel-silverstone-$(VERSION)-linux-$(ARCH)
PACKAGE_XCVR_CEL_SILVERSTONE_FILE := $(PACKAGE_ARTIFACTS_DIR)/$(PACKAGE_XCVR_CEL_SILVERSTONE_DIR).tar.gz
PACKAGE_FILES := $(PACKAGE_CORE_FILE) $(PACKAGE_LLDP_FILE) $(PACKAGE_XCVR_DELL_S5200_FILE) $(PACKAGE_XCVR_PDDF_FILE) $(PACKAGE_XCVR_CEL_SEASTONE2_FILE) $(PACKAGE_XCVR_CEL_SILVERSTONE_FILE)

# This snippet has been sourced and adapted from the following file which is
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
LIBSAI_DEB_FILES := $(DOWNLOAD_DIR)/$(BRCM_XGS_SAI_DEB) $(DOWNLOAD_DIR)/$(BRCM_XGS_SAI_DEV_DEB)
LIBSAI_FILES := $(MKFILE_DIR)/lib/libsai.so $(MKFILE_DIR)/lib/libsai.so.1 $(MKFILE_DIR)/lib/libsai.so.1.0

.PHONY: help
help: ## Display this help.
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_0-9-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

all: init build ## Runs 'init' and 'build' targets

init: download_libsai extract_libsai ## Ensures all dependencies for the project are in place, and downloads and extracts them if necessary

build: onie-sai onie-lldp  ## Builds the project

download_libsai: $(LIBSAI_DEB_FILES)

$(LIBSAI_DEB_FILES) &:
	wget -O $(DOWNLOAD_DIR)/$(BRCM_XGS_SAI_DEB) $(LIBSAIBCM_XGS_URL_PREFIX)/$(BRCM_XGS_SAI_DEB)
	wget -O $(DOWNLOAD_DIR)/$(BRCM_XGS_SAI_DEV_DEB) $(LIBSAIBCM_XGS_URL_PREFIX)/$(BRCM_XGS_SAI_DEV_DEB)

extract_libsai: $(LIBSAI_FILES)

$(LIBSAI_FILES) &: $(LIBSAI_DEB_FILES)
	mkdir -p $(SAIBCM_DIR) && cd $(SAIBCM_DIR) && \
	ar x $(DOWNLOAD_DIR)/$(BRCM_XGS_SAI_DEB) && \
	tar -xf data.tar.xz && \
	rm -f data.tar.xz control.tar.xz debian-binary && \
	ar x $(DOWNLOAD_DIR)/$(BRCM_XGS_SAI_DEV_DEB) && \
	tar -xf data.tar.xz && \
	rm -f data.tar.xz control.tar.xz debian-binary && \
	cp -Pav $(SAIBCM_DIR)/usr/lib/* $(MKFILE_DIR)/lib/ && \
	chmod -v +x $(MKFILE_DIR)/lib/*.so* && \
	touch $(MKFILE_DIR)/lib/*.so*

onie-sai: $(CARGO_TARGETS) ## Builds 'onie-sai' for the release target

onie-lldp: $(CARGO_TARGETS) ## Builds 'onie-lldp' for the release target

$(CARGO_TARGETS) &: $(SRC_FILES) $(LIBSAI_FILES)
	LD_LIBRARY_PATH="$(MKFILE_DIR)/lib:$$LD_LIBRARY_PATH" \
		cargo build --release --workspace

package: $(PACKAGE_FILES) ## Create release packages

package_core: $(PACKAGE_CORE_FILE)

$(PACKAGE_CORE_FILE): onie-sai
	cd $(PACKAGE_ARTIFACTS_DIR) && \
	mkdir $(PACKAGE_CORE_DIR) && cd $(PACKAGE_CORE_DIR) && \
	mkdir -vp usr/bin && \
	cp -v $(CARGO_TARGET_DIR)/release/onie-sai usr/bin/ && \
	ln -sv onie-sai usr/bin/onie-saictl && \
	ln -sv onie-sai usr/bin/xcvrctl && \
	ln -sv onie-sai usr/bin/onie-said && \
	cd $(PACKAGE_ARTIFACTS_DIR) && \
	tar -czvf $(PACKAGE_CORE_FILE) $(PACKAGE_CORE_DIR) && \
	rm -rf $(PACKAGE_CORE_DIR)

package_lldp: $(PACKAGE_LLDP_FILE)

$(PACKAGE_LLDP_FILE): onie-lldp
	cd $(PACKAGE_ARTIFACTS_DIR) && \
	mkdir $(PACKAGE_LLDP_DIR) && cd $(PACKAGE_LLDP_DIR) && \
	mkdir -vp usr/bin && \
	cp -v $(CARGO_TARGET_DIR)/release/onie-lldp usr/bin/ && \
	ln -sv onie-lldp usr/bin/onie-saictl && \
	ln -sv onie-lldp usr/bin/onie-said && \
	ln -sv onie-lldp usr/bin/onie-lldpctl && \
	ln -sv onie-lldp usr/bin/onie-lldpd && \
	cd $(PACKAGE_ARTIFACTS_DIR) && \
	tar -czvf $(PACKAGE_LLDP_FILE) $(PACKAGE_LLDP_DIR) && \
	rm -rf $(PACKAGE_LLDP_DIR)

package_xcvr_dell_s5200: $(PACKAGE_XCVR_DELL_S5200_FILE)

$(PACKAGE_XCVR_DELL_S5200_FILE): onie-sai
	cd $(PACKAGE_ARTIFACTS_DIR) && \
	mkdir $(PACKAGE_XCVR_DELL_S5200_DIR) && cd $(PACKAGE_XCVR_DELL_S5200_DIR) && \
	mkdir -vp usr/lib/platform && \
	cp -v $(CARGO_TARGET_DIR)/release/libxcvr_dell_s5200.so usr/lib/platform/ && \
	ln -sv libxcvr_dell_s5200.so usr/lib/platform/x86_64-dellemc_s5212f_c3538-r0.so && \
	ln -sv libxcvr_dell_s5200.so usr/lib/platform/x86_64-dellemc_s5224f_c3538-r0.so && \
	ln -sv libxcvr_dell_s5200.so usr/lib/platform/x86_64-dellemc_s5232f_c3538-r0.so && \
	ln -sv libxcvr_dell_s5200.so usr/lib/platform/x86_64-dellemc_s5248f_c3538-r0.so && \
	ln -sv libxcvr_dell_s5200.so usr/lib/platform/x86_64-dellemc_s5296f_c3538-r0.so && \
	cd $(PACKAGE_ARTIFACTS_DIR) && \
	tar -czvf $(PACKAGE_XCVR_DELL_S5200_FILE) $(PACKAGE_XCVR_DELL_S5200_DIR) && \
	rm -rf $(PACKAGE_XCVR_DELL_S5200_DIR)

package_xcvr_pddf: $(PACKAGE_XCVR_PDDF_FILE)

$(PACKAGE_XCVR_PDDF_FILE): onie-sai
	cd $(PACKAGE_ARTIFACTS_DIR) && \
	mkdir $(PACKAGE_XCVR_PDDF_DIR) && cd $(PACKAGE_XCVR_PDDF_DIR) && \
	mkdir -vp usr/lib/platform && \
	cp -v $(CARGO_TARGET_DIR)/release/libxcvr_pddf.so usr/lib/platform/ && \
	ln -sv libxcvr_pddf.so usr/lib/platform/x86_64-accton_as4630_54npe-r0.so && \
	ln -sv libxcvr_pddf.so usr/lib/platform/x86_64-accton_as4630_54pe-r0.so && \
	ln -sv libxcvr_pddf.so usr/lib/platform/x86_64-accton_as4630_54te-r0.so && \
	ln -sv libxcvr_pddf.so usr/lib/platform/x86_64-accton_as4630_54npem-r0.so && \
	ln -sv libxcvr_pddf.so usr/lib/platform/x86_64-accton_as7326_56x-r0.so && \
	ln -sv libxcvr_pddf.so usr/lib/platform/x86_64-accton_as7726_32x-r0.so && \
	cd $(PACKAGE_ARTIFACTS_DIR) && \
	tar -czvf $(PACKAGE_XCVR_PDDF_FILE) $(PACKAGE_XCVR_PDDF_DIR) && \
	rm -rf $(PACKAGE_XCVR_PDDF_DIR)

package_xcvr_cel_seastone2: $(PACKAGE_XCVR_CEL_SEASTONE2_FILE)

$(PACKAGE_XCVR_CEL_SEASTONE2_FILE): onie-sai
	cd $(PACKAGE_ARTIFACTS_DIR) && \
	mkdir $(PACKAGE_XCVR_CEL_SEASTONE2_DIR) && cd $(PACKAGE_XCVR_CEL_SEASTONE2_DIR) && \
	mkdir -vp usr/lib/platform && \
	cp -v $(CARGO_TARGET_DIR)/release/libxcvr_cel_seastone2.so usr/lib/platform/ && \
	ln -sv libxcvr_cel_seastone2.so usr/lib/platform/x86_64-cel_seastone_2-r0.so && \
	cd $(PACKAGE_ARTIFACTS_DIR) && \
	tar -czvf $(PACKAGE_XCVR_CEL_SEASTONE2_FILE) $(PACKAGE_XCVR_CEL_SEASTONE2_DIR) && \
	rm -rf $(PACKAGE_XCVR_CEL_SEASTONE2_DIR)

package_xcvr_cel_silverstone: $(PACKAGE_XCVR_CEL_SILVERSTONE_FILE)

$(PACKAGE_XCVR_CEL_SILVERSTONE_FILE): onie-sai
        cd $(PACKAGE_ARTIFACTS_DIR) && \
        mkdir $(PACKAGE_XCVR_CEL_SILVERSTONE_DIR) && cd $(PACKAGE_XCVR_CEL_SILVERSTONE_DIR) && \
        mkdir -vp usr/lib/platform && \
        cp -v $(CARGO_TARGET_DIR)/release/libxcvr_cel_silverstone.so usr/lib/platform/ && \
        ln -sv libxcvr_cel_silverstone.so usr/lib/platform/x86_64-cel_silverstone-r0.so && \
        cd $(PACKAGE_ARTIFACTS_DIR) && \
        tar -czvf $(PACKAGE_XCVR_CEL_SILVERSTONE_FILE) $(PACKAGE_XCVR_CEL_SILVERSTONE_DIR) && \
        rm -rf $(PACKAGE_XCVR_CEL_SILVERSTONE_DIR)

clean: cargo-clean ## Cleans the project directory

all-clean: cargo-clean package-clean lib-clean download-clean ## Cleans the whole project directory, and deletes downloaded dependencies, and generated packages

.PHONY: cargo-clean
cargo-clean: ## Cleans the whole target/ directory
	cargo clean || true

.PHONY: lib-clean
lib-clean: ## Cleans the lib/ directory
	rm -rvf $(MKFILE_DIR)/lib/*.so* || true

.PHONY: download-clean
download-clean: ## Cleans the dl/ directory
	rm -rvf $(DOWNLOAD_DIR)/* || true

.PHONY: package-clean
package-clean: ## Cleans all generated packages
	rm -rvf $(PACKAGE_FILES) || true
