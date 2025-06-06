"""
Setup script for DiskDB Python client
"""

from setuptools import setup, find_packages
import os

# Read the README file
here = os.path.abspath(os.path.dirname(__file__))
with open(os.path.join(here, 'README.md'), encoding='utf-8') as f:
    long_description = f.read()

setup(
    name='diskdb',
    version='0.1.0',
    author='DiskDB Team',
    author_email='team@diskdb.io',
    description='Python client for DiskDB - A fast, persistent key-value database',
    long_description=long_description,
    long_description_content_type='text/markdown',
    url='https://github.com/yourusername/DiskDB',
    project_urls={
        'Bug Reports': 'https://github.com/yourusername/DiskDB/issues',
        'Source': 'https://github.com/yourusername/DiskDB',
        'Documentation': 'https://diskdb.readthedocs.io',
    },
    packages=find_packages(),
    classifiers=[
        'Development Status :: 4 - Beta',
        'Intended Audience :: Developers',
        'Topic :: Database',
        'Topic :: Software Development :: Libraries :: Python Modules',
        'License :: OSI Approved :: MIT License',
        'Programming Language :: Python :: 3',
        'Programming Language :: Python :: 3.7',
        'Programming Language :: Python :: 3.8',
        'Programming Language :: Python :: 3.9',
        'Programming Language :: Python :: 3.10',
        'Programming Language :: Python :: 3.11',
        'Programming Language :: Python :: 3.12',
        'Operating System :: OS Independent',
    ],
    python_requires='>=3.7',
    install_requires=[],  # No external dependencies!
    extras_require={
        'dev': [
            'pytest>=6.0',
            'pytest-asyncio',
            'black',
            'flake8',
            'mypy',
        ],
    },
    keywords='database key-value redis-compatible persistent diskdb rocksdb',
)