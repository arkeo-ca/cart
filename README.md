# CaRT (Compressed and RC4 Transport)
This is a Rust port of the original CaRT library/utility created by the Government of Canada.
While the library was completely rewritten, it attempts to maintain the same interface as the original
work, where possible. The original project, written in Python, can be found [here](https://bitbucket.org/cse-assemblyline/cart/).

The CaRT file format is used to store/transfer malware and its associated metadata. 
It neuters the malware so it cannot be executed and encrypts it so 
anti-virus softwares cannot flag the CaRT file as malware.

## Advantages

* FAST: CaRT is just as fast as zipping a file
* STREAMING: CaRT uses zlib and RC4 which allow it to encode files in streaming
* METADATA: CaRT can store the file metadata in the same file as the file itself; the metadata can be read without 
reading the full file
* HASH CALCULATION: CaRT calculates the hashes of the file while it is encoding it and store that information in the 
footer
* SIZE: CaRT files are usually smaller then the original files because it uses compression. (Except in the case we you 
store huge amount of metadata in the CaRT)

## Using CaRT with STIX v2
Now that stix v2 uses JSON as encoding, you can now bundle your STIX report directly in the CaRT format. When CaRT encode files, 
it add metadata from *.cartmeta file with the same prefix of your file. Therefore, if you save your STIX report to a .cartmeta file, 
the resulting CaRT file will have the full STIX report embedded with it.

Example:

    $ ls
      file.exe            - File I want to encode
      file.exe.cartmeta   - Stix report of file.exe

    $ cart file.exe
    $ ls
      file.exe            - File I want to encode
      file.exe.cartmeta   - Stix report of file.exe
      file.exe.cart       - CaRT file containig both the file.exe and it's stix report

## Format Overview

### Mandatory Header (38 bytes)

CaRT has a mandatory header that looks like this

     4s     h         Q        16s         Q
    CART<VERSION><RESERVED><ARC4KEY><OPT_HEADER_LEN>
    
Where VERSION is 1 and RESERVED is 0. It most cases, the RC4 key used to decrypt the file is stored in the mandatory 
header and is always the same one (first 8 digit of pi twice). In addition, CaRT provides an option to override the key 
which then stores null bytes in the mandatory header. You'll then need to know the key to unCaRT the file...

### Optional Header (OPT_HEADER_LEN bytes)

CaRT's optional header is a OPT_HEADER_LEN bytes RC4 blob of a json serialized header

    RC4(<JSON_SERIALIZED_OPTIONAL_HEADER>)

### Data block (N Bytes)

CaRT's data block is a zlib then RC4 block 

    RC4(ZLIB(block encoded stream))

### Optional Footer (OPT_FOOTER_LEN bytes)

Like the optional header, CaRT's optional footer is a OPT_FOOTER_LEN bytes RC4 blob of a json serialized footer

    RC4(<JSON_SERIALIZED_OPTIONAL_FOOTER>)

###  Mandatory Footer (28 Bytes)

CaRT ends its file with a mandatory footer which allow the format to read the footer and return the hashes without reading the whole file

     4s      Q           Q               Q
    TRAC<RESERVED><OPT_FOOTER_POS><OPT_FOOTER_LEN>

## Command line interface 

By installing the crate, you get access to the CaRT library and the CaRT CLI. 

The CaRT CLI has the following priority for its options:

* There are defaults values for all the options inside the CLI
* Default values are overridden by options in ~/.cart/cart.cfg 
* Values in the configuration file are overridden by CLI options

These are the options available in the CaRT CLI:

    Usage: cart [options] file1 file2 ... fileN
    
    Options:
      --version             show program's version number and exit
      -h, --help            show this help message and exit
      -f, --force           Replace output file if it already exists
      -i, --ignore          Ignore RC4 key from conf file
      -j JSONMETA, --jsonmeta=JSONMETA
                            Provide header metadata as json blob
      -k KEY, --key=KEY     Use private RC4 key (base64 encoded). Same key must be
                            provided to unCaRT.
      -m, --meta            Keep metadata around when extracting CaRTs
      -n FILENAME, --name=FILENAME
                            Use this value as metadata filename
      -o OUTFILE, --outfile=OUTFILE
                            Set output file
      -s, --showmeta        Only show the file metadata

The CaRT configuration file look like this:

    [global]
    # rc4_key is a base64 representation of your key
    rc4_key: AvUzYXNkZg==
    # keep_meta is an equivalent to -m in the CLI
    keep_meta: True
    # force is an equivalent to -f in the CLI
    force: True
    
    # default_header is a key/value pair of data to be added to the CaRT in the optional header
    [default_header]
    poc: Your Name
    poc_email: your.name@your.org
