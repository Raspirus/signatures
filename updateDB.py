#!/usr/bin/python
from modules.malwarebazaar import MalwareBazaar
from modules.virusshare import VirusShare
mb = MalwareBazaar()
# mb.download()
mb.loadHashes()
mb.merge()
vs = VirusShare()
# vs.download()
vs.loadHashes()
vs.merge()
