#!/usr/bin/env python3

#  This Source Code Form is subject to the terms of the Mozilla Public
#  License, v. 2.0. If a copy of the MPL was not distributed with this
#  file, You can obtain one at http://mozilla.org/MPL/2.0/.

from pathlib import Path
from platform import system
from setuptools import Extension, find_packages, setup

compile_args = []
if system() != "Windows":
    compile_args.extend(["-std=c99", "-Wno-unused-variable"])

setup(
    name="tree-sitter-minizinc",
    use_scm_version=True,
    setup_requires=["setuptools_scm"],
    python_requires=">=3.6",
    author="Jip J. Dekker",
    author_email="jip.dekker@monash.edu",
    description="",
    long_description=Path("README.md").read_text(encoding="UTF-8"),
    long_description_content_type="text/markdown",
    url="https://www.minizinc.org/",
    project_urls={
        "Bug Tracker": "https://github.com/Dekker1/tree-sitter-minizinc/issues",
        "Source": "https://github.com/Dekker1/tree-sitter-minizinc",
    },
    packages=find_packages(where="bindings/python"),
    package_dir={"": "bindings/python"},
    ext_modules=[
        Extension(
            "tree_sitter_minizinc.binding",
            ["src/parser.c"],
            include_dirs=["src"],
            extra_compile_args=compile_args,
        )
    ],
    classifiers=[
        "Development Status :: 4 - Beta",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.6",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: Implementation :: CPython",
        "Programming Language :: Python :: Implementation :: PyPy",
        "License :: OSI Approved :: Mozilla Public License 2.0 (MPL 2.0)",
        "Operating System :: OS Independent",
        "Topic :: Scientific/Engineering :: Artificial Intelligence",
        "Topic :: Scientific/Engineering :: Mathematics",
    ],
    install_requires=["tree-sitter>= 0.19"],
    entry_points="""
        [pygments.lexers]
        minizinclexer = tree_sitter_minizinc:MiniZincLexer
    """,
    package_data={"": ["queries/*.scm"]},
)
