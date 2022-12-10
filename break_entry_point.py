#!/bin/env/python3

import gdb
import sys
from os import path
from target import gdb_import as conf

if(conf.kernel_path is None):
  print("must specify kernel binary path")

if(path.exists(conf.kernel_path)):
  print("kernel binary is not found at %s" % (conf.kernel_path))

# Yes/Noを聞いてくるコマンドで指示を出さなくてよくなる
gdb.execute('set confirm off')

# remoteのqemuにアクセス
gdb.execute('target remote localhost:9000')

# kernelのバイナリを読み込んでシンボルを扱えるようにする
# 一旦0x80_0000_0000をオフセットに設定しているが
# 本当はbootloaderの中で決定するオフセットの値を確認してから設定したい。
gdb.execute('add-symbol-file %s -o 0x8000000000' % (conf.kernel_path))

gdb.execute('b kernel_main')

