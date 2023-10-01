#!/usr/bin/python
from modules.malwarebazaar import MalwareBazaar
from modules.virusshare import VirusShare

# This script will fetch Hashes from various sources and put them in this repository
# The script will be executed through a GitHub Action.
# Each Source of Hashes will have its own function that might also be run in parallel. This helps us determine any issues that might arise
# This script should also automatically skip Hashes that have been reported as false positives. These hashes will be stored in a separate text file
# On update, the script should not download all hashes again, but instead only add the new ones.
# Once executed, the script should report the total amount of hashes and the date it was last updated. This information can be updated in the README.md file


mb = MalwareBazaar()
mb.download()
mb.loadHashes()
mb.merge()
#vs = VirusShare()
# vs.download()
#vs.loadHashes()
#vs.merge()
