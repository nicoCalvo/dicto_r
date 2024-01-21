import json

from  dicto_r import dictor

with open("basic.json") as data:
        BASIC = json.load(data)


# def main():
    
#     res = dictor({"pepe": {"pito":{"poto": [1,2,3]}}}, "pepe.pito.poto.4")


def test_simple_dict():
    """test for value in a dictionary"""
    result = dictor(BASIC, "robocop.year")
    assert result == 1989

def test_non_existent_value():
    """test a non existent key search"""
    result = dictor(BASIC, "non.existent.value")
    assert result is None

    result = dictor({"lastname": "Doe"}, "foo.lastname")
    assert result is None

def test_zero_value():
    """test a Zero value - should return 0"""
    result = dictor(BASIC, "terminator.2.terminator 3.year", checknone=True)
    assert result == 0

def test_partial_exist_value():
    """partially existing value"""
    result = dictor(BASIC, "spaceballs.year.fakekey")
    assert result is None

def test_random_chars():
    result = dictor(BASIC, "#.random,,,@.chars")
    assert result == '({%^&$"'

def test_complex_dict():
    """test parsing down a list and getting dict value"""
    result = dictor(BASIC, "terminator.1.terminator 2.genre.0")
    assert result == "nuclear war"

def test_pathsep():
    """test parsing down a list and getting dict value with pathsep"""
    result = dictor(BASIC, "terminator/1/terminator 2/genre/0", pathsep="/")
    assert result == "nuclear war"

def test_keys_with_different_pathsep():
    """test parsing keys with different path separator"""
    result = dictor(BASIC, "dirty.harry/genre", pathsep="/")
    assert result == "romance"

def test_escape_pathsep():
    """test using escape path separator"""
    result = dictor(BASIC, "dirty\.harry.genre")
    assert result == "romance"

def test_ignore_letter_casing():
    """test ignoring letter upper/lower case"""
    result = dictor(BASIC, "austin PoWeRs.year", ignorecase=True)
    assert result == 1996

def test_ignore_letter_casing_nested():
    """test ignoring letter upper/lower case"""
    result = dictor(BASIC, "austin PoWeRs.Year", ignorecase=True)
    assert result == 1996

def test_numeric_key_handling():
    """test handling keys that are numbers"""
    result = dictor(BASIC, "1492.year")
    assert result == 1986