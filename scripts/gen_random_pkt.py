import random
import string

N_LINES = 10
OUTPUT_FILENAME = "fuzz.pkt"
SEED = None  # set to a number for a fixed seed

MAX_FILE_SIZE_IN_KB = None  # this assumes that every character is 1b

MAX_COMMENT_LENGTH = 100
MAX_IDENTIFIER_LENGTH = 100
MAX_WHITESPACE_LENGTH = 10
MAX_WHITESPACE_WITH_LINES_LENGTH = 2
WHITESPACE_WEIGHTS = [0.7, 0.3]  # space, newline

MAX_DEPTH = 3
MAX_TUPLE_ELEMENTS = 100
MAX_ARRAY_DIMENSIONS = 10


PKT_TYPES = ["uint8", "uint16", "uint32", "int8", "int16", "int32", "float"]


def identifier() -> str:
    return random.choice(string.ascii_letters + "_") + "".join(
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


def generate_non_array_type(depth: int = 0) -> str:
    kind = random.choice(["number", "flag", "tuple"])
    if kind == "tuple" and depth < MAX_DEPTH:
        op = "(" + __()
        cl = ")" + __()
        tuples = ",".join(
            [
                generate_type(depth + 1) + whitespace_with_newlines()
                for _ in range(random.randint(1, MAX_TUPLE_ELEMENTS))
            ]
        )
        return f"{identifier()}:{_w()}{op}{tuples}{cl}"
    elif kind == "number" or depth >= MAX_DEPTH:
        return f"{identifier()}:{_w()}{random.choice(PKT_TYPES)}"
    elif kind == "flag":
        variant_s = ",".join(
            identifier().upper() + whitespace_with_newlines()
            for _ in range(random.randint(1, 3))
        )
        op = "{" + __()
        cl = "}" + __()
        return f"{identifier()}:{_w()}{op}{variant_s}{cl}"
    else:
        raise RuntimeError(f"Unknown kind `{kind}`")


def generate_type(depth: int = 0) -> str:
    if depth > MAX_DEPTH:
        return ""

    ty = generate_non_array_type(depth).strip()
    if random.random() < 0.25:
        ty += "[]" * random.randint(1, MAX_ARRAY_DIMENSIONS)
    return ty


def generate_line() -> str:
    if random.random() < 1 / 6:
        return "#" + "".join(
            random.choice(string.ascii_letters + string.digits + " \t")
            for _ in range(random.randint(0, MAX_COMMENT_LENGTH))
        )
    return generate_type()


if __name__ == "__main__":
    if SEED is not None:
        random.seed(SEED)

    total_size = 0
    with open(OUTPUT_FILENAME, "w") as f:
        for _ in range(N_LINES):
            if max_size := MAX_FILE_SIZE_IN_KB and total_size / 1024 >= max_size:
                break
            line = generate_line() + "\n"
            total_size += len(line)
            f.write(line)
