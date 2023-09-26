#!/bin/python
from modules.downloader import Downloader
import requests

class VirusShare(Downloader):
	def __init__(self,):
		super().__init__("VirusShare", "https://virusshare.com/hashfiles/")
	def download(self): # TODO: Maybe add a "force" option to always redownload the list?
		with open("full_virusshare.txt", 'w') as vsFile:
			for i in range(0,480):
				r = requests.get(self.url + f"VirusShare_{'%05d' % i}.md5")
				vsFile.write(r.text[198:])
	def loadHashes(self):
		with open("full_virusshare.txt", 'r') as vsFile:
			self.lines = vsFile.readlines()
			# for i in range(1_200_000):
			# 	self.lines.append(vsFile.readline())
	def readHashFile(self):
		self.outFileModified = 0
		try:
			with open(f"raspirus_{'%03d' % self.fileNum}.md5", 'r') as f:
				lines = f.readlines()
				self.md5 = [int(x, 16) for x in lines]
		except FileNotFoundError:
			with open(f"raspirus_{'%03d' % self.fileNum}.md5", 'w') as f:
				pass
			self.md5 = []

