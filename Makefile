SRC_DIR := src
OBJ_DIR := obj
BIN_DIR := bin

EXE := $(BIN_DIR)/saictl
SRC := $(wildcard $(SRC_DIR)/*.c)
OBJ := $(SRC:$(SRC_DIR)/%.c=$(OBJ_DIR)/%.o)

CXXFLAGS ?=
CXXFLAGS += -ansi
CXXFLAGS += -fPIC
CXXFLAGS += -pipe
CXXFLAGS += -std=c++14
CXXFLAGS += -Wall
CXXFLAGS += -Wcast-align
CXXFLAGS += -Wcast-qual
CXXFLAGS += -Wconversion
CXXFLAGS += -Wdisabled-optimization
#CXXFLAGS += -Werror
CXXFLAGS += -Wextra
CXXFLAGS += -Wfloat-equal
CXXFLAGS += -Wformat=2
CXXFLAGS += -Wformat-nonliteral
CXXFLAGS += -Wformat-security
CXXFLAGS += -Wformat-y2k
CXXFLAGS += -Wimport
CXXFLAGS += -Winit-self
CXXFLAGS += -Wno-inline
CXXFLAGS += -Winvalid-pch
CXXFLAGS += -Wmissing-field-initializers
CXXFLAGS += -Wmissing-format-attribute
CXXFLAGS += -Wmissing-include-dirs
CXXFLAGS += -Wmissing-noreturn
CXXFLAGS += -Wno-aggregate-return
CXXFLAGS += -Wno-padded
CXXFLAGS += -Wno-switch-enum
CXXFLAGS += -Wno-unused-parameter
CXXFLAGS += -Wpacked
CXXFLAGS += -Wpointer-arith
CXXFLAGS += -Wredundant-decls
CXXFLAGS += -Wshadow
CXXFLAGS += -Wstack-protector
CXXFLAGS += -Wstrict-aliasing=3
CXXFLAGS += -Wswitch
CXXFLAGS += -Wswitch-default
CXXFLAGS += -Wunreachable-code
CXXFLAGS += -Wunused
CXXFLAGS += -Wvariadic-macros
CXXFLAGS += -Wwrite-strings
CXXFLAGS += -Wno-switch-default
CXXFLAGS += -Wconversion
CXXFLAGS += -Wno-psabi
CXXFLAGS += -Wcast-align=strict
CXXFLAGS += -Xlinker --no-as-needed

CPPFLAGS := -Iinclude -Iinclude/sai -Iinclude/sai/experimental
CFLAGS   := -g -Wall -Werror -Wno-error=unused-function -fPIC -pipe -Xlinker --no-as-needed
LDFLAGS  := -Llib
LDLIBS   := -lsai

.PHONY: all clean

all: $(EXE)

$(EXE): $(OBJ) | $(BIN_DIR)
	$(CC) $(CFLAGS) $(LDFLAGS) $^ $(LDLIBS) -o $@

$(OBJ_DIR)/%.o: $(SRC_DIR)/%.c | $(OBJ_DIR)
	$(CC) $(CPPFLAGS) $(CFLAGS) -c $< -o $@

$(BIN_DIR) $(OBJ_DIR):
	mkdir -p $@

clean:
	@$(RM) -rv $(BIN_DIR) $(OBJ_DIR)

-include $(OBJ:.o=.d)
