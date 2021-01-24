import random
import string
from typing import Set

N_DEFNS = 100
OUTPUT_FILENAME = "fuzz.pkt"
SEED = None  # set to a number for a fixed seed

MAX_FILE_SIZE_IN_KB = 64  # this assumes that every character is 1b

MAX_COMMENT_LENGTH = 100
MAX_IDENTIFIER_LENGTH = 100
MAX_WHITESPACE_LENGTH = 10
MAX_WHITESPACE_WITH_LINES_LENGTH = 2
WHITESPACE_WEIGHTS = [0.7, 0.3]  # space, newline

MAX_DEPTH = 3
MAX_ENUM_VARIANTS = 32
MAX_ARRAY_DIMENSIONS = 1
MAX_STRUCT_FIELDS = 100
# Use `min(available, MAX_STRUCT_FIELDS)` if bounded, `available` otherwise
STRUCT_FIELDS_MODE = ("bounded", "unbounded")[1]


PKT_TYPES = ["string", "uint8", "uint16", "uint32", "int8", "int16", "int32", "float"]


def identifier() -> str:
    return random.choice(string.ascii_lowercase + "_") + "".join(
        random.choice(string.ascii_letters + "_" + string.digits)
        for _ in range(random.randint(1, MAX_IDENTIFIER_LENGTH))
    )


def whitespace() -> str:
    return " " * random.randint(0, MAX_WHITESPACE_LENGTH)


def whitespace_with_newlines() -> str:
    return "".join(
        random.choices([" ", "\n"], WHITESPACE_WEIGHTS)[0]
        for _ in range(random.randint(0, MAX_WHITESPACE_WITH_LINES_LENGTH))
    )


_w = whitespace
__ = whitespace_with_newlines


def generate_type(types: Set[str]) -> str:
    kind = random.choice(["enum", "struct"])
    op = "{" + __()
    cl = "}" + __()
    if kind == "enum":
        body = ",".join(
            [
                identifier() + whitespace_with_newlines()
                for _ in range(random.randint(1, MAX_ENUM_VARIANTS))
            ]
        )
    elif kind == "struct":
        if STRUCT_FIELDS_MODE == "bounded":
            n_fields = min(random.randint(1, MAX_STRUCT_FIELDS), len(types))
            fields = random.sample(tuple(types), n_fields)
        else:
            fields = types
        body = ",\n".join(
            "    " + identifier().upper() + ": " + field + maybe_array()
            for field in fields
        )
    else:
        raise RuntimeError(f"Unknown kind `{kind}`")

    name = identifier().capitalize()
    types.add(name)
    return f"{name}:{_w()}{kind}{_w()}{op}{body}{cl}"


def maybe_array() -> str:
    if random.random() < 0.25:
        return "[]" * random.randint(1, MAX_ARRAY_DIMENSIONS)
    return ""


def generate_definition(types: Set[str]) -> str:
    if random.random() < 1 / 6:
        return "#" + "".join(
            random.choice(string.ascii_letters + string.digits + " \t")
            for _ in range(random.randint(0, MAX_COMMENT_LENGTH))
        )
    return generate_type(types)


if __name__ == "__main__":
    if SEED is not None:
        random.seed(SEED)

    total_size = 0
    with open(OUTPUT_FILENAME, "w") as f:
        types = set(PKT_TYPES)
        for _ in range(N_DEFNS):
            if (max_size := MAX_FILE_SIZE_IN_KB) and (total_size / 1024 >= max_size):
                break
            line = generate_definition(types) + "\n"
            total_size += len(line)
            f.write(line)

        f.write("\n export " + random.sample(tuple(types - set(PKT_TYPES)), 1)[0])
