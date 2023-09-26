#!/usr/bin/python
from tqdm import tqdm
from bisect import bisect_left

class Downloader:
	hashesPerFile = 1_000_000
	def __init__(self, name, url):
		self.name = name
		self.url = url
		self.lines = []
	
	def merge(self):
		self.fileNum = 0
		self.readHashFile()
		outfile = open(f"raspirus_{'%03d' % self.fileNum}.md5", 'w')
		for line in tqdm(self.lines):
			line = int(line, 16, unsigned=True)
			while(1):
				if (len(self.md5) < Downloader.hashesPerFile):
					# bisect_left will return the position of the md5 if it's contained and where it would go if it isn't.
					# the value of the returned position has to be checked to see if line is contained in md5 or not
					pos = bisect_left(self.md5, line, 0, None)
				else:
					self.fileNum += 1
					self.readHashFile()
					continue
				try:
					if (self.md5[pos] != line):
						self.md5.insert(pos, line)
						self.outFileModified = 1
						if(len(self.md5) == Downloader.hashesPerFile):
							outfile.writelines([f"{x:0>32x}\n" for x in self.md5])
							self.fileNum += 1
							outfile.close()
							outfile = open(f"raspirus_{'%03d' % self.fileNum}.md5", 'w')
							self.readHashFile()
						break
					else:
						break
				except IndexError:
					self.md5.insert(pos, line)
					self.outFileModified = 1
					if(len(self.md5) == Downloader.hashesPerFile):
						outfile.writelines([f"{x:0>32x}\n" for x in self.md5])
						self.fileNum += 1
						outfile.close()
						outfile = open(f"raspirus_{'%03d' % self.fileNum}.md5", 'w')
						self.readHashFile()
					break
		# Write any remaining hashes
		if self.outFileModified:
			outfile.writelines([f"{x:0>32x}\n" for x in self.md5])
		outfile.close()
