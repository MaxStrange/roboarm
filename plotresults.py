"""
Plots the results of an experiment by parsing a results.txt file.
"""
import matplotlib.pyplot as plt
import os
import sys

class Episode:
    def __init__(self, lines):
        self.episode_number = int(lines[0].strip().split(' ')[-1])

    @staticmethod
    def present_in_contents(lines):
        """
        Returns if the given lines contains a line that starts with 'episode '.
        Lines must be a list of strings.
        """
        for line in lines:
            if line.strip().lower().startswith("episode "):
                return True
        return False

    @staticmethod
    def parse_from_contents(lines):
        """
        Removes the lines needed to parse the episode and parses it.
        """
        ended = False
        parse_mode = True
        for i, line in enumerate(lines):
            if line.lower().startswith("episode ") and parse_mode:
                start_line = i
                parse_mode = False
            elif line.lower().startswith("episode ") and not parse_mode:
                end_line = i
                ended = True
                break
        if not ended:
            end_line = len(lines)

        genetic = False
        for line in lines:
            if "network" in line.lower():
                genetic = True
                break
        if genetic:
            return GeneticEpisode(lines[start_line:end_line]), lines[end_line:]
        else:
            return RandomEpisode(lines[start_line:end_line]), lines[end_line:]

class Network:
    def __init__(self, lines):
        self.fitness = None
        self.index = None
        self.servos = {}
        have_seen_network = False
        for line in lines:
            if line.lower().startswith("network ") and not have_seen_network:
                self.index = int(line.strip().split(' ')[-1])
                have_seen_network = True
            elif line.lower().startswith("network ") and have_seen_network:
                assert False, "Found a second network index in this network's lines for parsing."
            elif line.lower().startswith("servo "):
                _, number, value = line.lower().strip().split(' ')
                if int(number) in self.servos:
                    self.servos[int(number)].append(float(value))
                else:
                    self.servos[int(number)] = [float(value)]
            elif line.lower().startswith("fitness "):
                self.fitness = float(line.strip().split(' ')[-1])
                break

        assert self.index != None, "Could not find an index for this network"
        assert self.fitness != None, "Could not find a fitness for network index {}".format(self.index)

    @staticmethod
    def present_in_contents(lines):
        for line in lines:
            if "network " in line.lower():
                return True
        return False

    @staticmethod
    def parse_from_contents(lines):
        ended = False
        parse_mode = True
        for i, line in enumerate(lines):
            if line.lower().startswith("network ") and parse_mode:
                start_line = i
                parse_mode = False
            elif line.lower().startswith("network ") and not parse_mode:
                end_line = i
                ended = True
                break

        if not ended:
            end_line = len(lines)
        return Network(lines[start_line:end_line]), lines[end_line:]

class GeneticEpisode(Episode):
    def __init__(self, lines):
        super().__init__(lines)
        servo_starting_values = {}
        self.networks = []
        # Parse out the starting values
        for line in lines:
            if line.lower().startswith("network"):
                break
            elif line.lower().startswith("servo "):
                _, number, value = line.lower().strip().split(' ')
                servo_starting_values[int(number)] = float(value)

        # Parse out each network
        while Network.present_in_contents(lines):
            network, lines = Network.parse_from_contents(lines)
            self.networks.append(network)

        assert self.networks, "Could not find any networks in GeneticEpisode {}".format(self.episode_number)
        print("Parsed GeneticEpisode {} of length {} networks, each of length {}".format(
            self.episode_number, len(self.networks), len(self.networks[0].servos[0])
        ))

class RandomEpisode(Episode):
    def __init__(self, lines):
        super().__init__(lines)
        self.servos = {}
        for line in lines:
            if line.lower().strip().startswith("servo "):
                _, number, value = line.lower().strip().split(' ')
                if int(number) in self.servos:
                    self.servos[int(number)].append(float(value))
                else:
                    self.servos[int(number)] = [float(value)]

        # Assert that this episode contains at least one recording
        self.servo_ids = [k for k in self.servos.keys()]
        self.servo_ids.sort()
        assert self.servo_ids, "Could not find any servos in RandomEpisode {}.".format(self.episode_number)

        # Assert that this episode contains the same number of recordings for each servo
        nservos = len(self.servos[self.servo_ids[0]])
        for id in self.servo_ids:
            assert len(self.servos[id]) == nservos, "Number of recordings is not the same for all servos in RandomEpisode {}.".format(self.episode_number)
        print("Parsed RandomEpisode {} of length {}".format(self.episode_number, nservos))

class ExperimentResults:
    def __init__(self, path):
        """
        Parses the given file path into an Experiment instance.
        """
        self.type = None
        self.episodes = []

        with open(path) as f:
            lines = [line for line in f]

        while Episode.present_in_contents(lines):
            ep, lines = Episode.parse_from_contents(lines)
            self.episodes.append(ep)

        if self.episodes:
            self.type = type(self.episodes[0])

    def __str__(self):
        return "Experiment is of type {} and consists of {} episodes.".format(self.type, len(self.episodes))

if __name__ == "__main__":
    if len(sys.argv) != 2 and not os.path.exists("results.txt"):
        print("Need a path to results.txt file.")
        exit(1)
    elif not os.path.exists(sys.argv[1]):
        print("{} does not exist.".format(sys.argv[1]))
        exit(1)

    if len(sys.argv) == 2:
        path = sys.argv[1]
    else:
        path = "results.txt"

    experiment = ExperimentResults(path)

    # Summarize the parsed experiment
    print(experiment)
