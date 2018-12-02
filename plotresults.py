"""
Plots the results of an experiment by parsing a results.txt file.
"""
import enum
import numpy as np
import matplotlib.pyplot as plt
import os
import sys

class ExperimentType(enum.Enum):
    RANDOM = 0
    GENETIC = 1

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

        # Assert that this network contains at least one recording
        self.servo_ids = [k for k in self.servos.keys()]
        self.servo_ids.sort()
        self.nservos = len(self.servo_ids)
        assert self.servo_ids, "Could not find any servos in Network {}.".format(self.index)

        # Assert that this network contains the same number of recordings for each servo
        self.nsteps = len(self.servos[self.servo_ids[0]])
        for id in self.servo_ids:
            assert len(self.servos[id]) == self.nsteps, "Number of recordings is not the same for all servos in Network {}.".format(self.index)

    @property
    def data(self):
        """
        Returns a Numpy Array of shape (nservos, nsteps).
        """
        servo_data = []
        for servo in self.servo_ids:
            servo_data.append(self.servos[servo])
        ret = np.vstack(servo_data)
        assert ret.shape == (self.nservos, self.nsteps)
        return ret

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
        self.nsteps = self.networks[0].nsteps
        for net in self.networks:
            assert net.nsteps == self.nsteps, "Not all networks in GeneticEpisode {} have the same number of steps. Network {} has {} but should have {}".format(
                self.episode_number, net.index, net.nsteps, self.nsteps
            )
        self.nservos = self.networks[0].nservos
        self.nnetworks = len(self.networks)
        self.fitnesses = {net.index: net.fitness for net in self.networks}
        self.highest_fitness = max(self.fitnesses.values())
        print("Parsed GeneticEpisode (Generation) {} of length {} networks, each evaluated on {} steps.".format(
            self.episode_number, self.nnetworks, self.nsteps
        ))

    @property
    def data(self):
        """
        Returns a Numpy Array of shape (nservos, nsteps, nnetworks).
        Since the networks do not actually have continuity across generations, it probably makes
        more sense to call episode.data_sorted_by_fitness instead of this.
        """
        netdata = []
        for network in self.networks:
            netdata.append(network.data)
        ret = np.array(netdata).reshape((self.nservos, self.nsteps, self.nnetworks))
        return ret

    @property
    def data_sorted_by_fitness(self):
        """
        Returns a Numpy Array of shape (nservos, nsteps, nnetworks) just like self.data, but
        rather than aligning the matrices of (servos, steps) to networks based simply on the
        network indexes, we align them based on the fitness of each network, so that
        the resulting data array is organized by most fit to least fit in the third dimension.
        """
        netids = [i for i in range(self.nnetworks)]
        network_ids_sorted_by_fitness = sorted(list(netids), key=lambda id: self.fitnesses[id], reverse=True)
        netdata = []
        for networkid in network_ids_sorted_by_fitness:
            netdata.append(self.networks[networkid].data)
        ret = np.array(netdata).reshape((self.nservos, self.nsteps, self.nnetworks))
        assert ret.shape == (self.nservos, self.nsteps, self.nnetworks)
        return ret

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
        self.nservos = len(self.servo_ids)
        assert self.servo_ids, "Could not find any servos in RandomEpisode {}.".format(self.episode_number)

        # Assert that this episode contains the same number of recordings for each servo
        self.nsteps = len(self.servos[self.servo_ids[0]])
        for id in self.servo_ids:
            assert len(self.servos[id]) == self.nsteps, "Number of recordings is not the same for all servos in RandomEpisode {}.".format(self.episode_number)
        print("Parsed RandomEpisode {} of length {}".format(self.episode_number, self.nsteps))

    @property
    def data(self):
        """
        Returns this Episode's data in the form of a NumpyArray of shape (nservos, nsteps)
        """
        arrays = []
        for servo in self.servo_ids:
            arr = np.array(self.servos[servo])
            arrays.append(arr)
        ret = np.vstack(arrays)
        assert ret.shape == (len(self.servo_ids), self.nsteps)
        return ret

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
            if type(self.episodes[0]) == RandomEpisode:
                self.type = ExperimentType.RANDOM
            elif type(self.episodes[0]) == GeneticEpisode:
                self.type = ExperimentType.GENETIC
            else:
                assert False, "ExperimentResults does not understand the type {}".format(type(self.episodes[0]))

        self.nepisodes = len(self.episodes)
        self.nservos = self.episodes[0].nservos
        self.nsteps_per_episode = self.episodes[0].nsteps
        self.nnetworks = self.episodes[0].nnetworks if self.type == ExperimentType.GENETIC else None

    def __str__(self):
        return "Experiment is of type {} and consists of {} episodes.".format(self.type, len(self.episodes))

    @property
    def data(self):
        """
        For a RANDOM Experiment:
        Returns all the servo values for each servo and episode in this experiment
        in the form of a Numpy Array of shape (nservos, nsteps_per_episode * nepisodes).

        For a GENETIC Experiment:
        Returns a tuple of the form (fitness_data, servo_data), where:

        - fitness_data is a list of length ngenerations (just nepisodes), where each value in the list is
          the fitness value for the best network in that generation
        - servo_data is a Numpy Array of shape (nservos, nsteps_per_generation, nnetworks, ngenerations).
          Every generation's networks are sorted by most fit to least fit before combining with the rest of the data.

        """
        if self.type == ExperimentType.RANDOM:
            # Compile data for random experiment type
            eparrays = []
            for ep in self.episodes:
                eparrays.append(ep.data)
            res = np.hstack(eparrays)
            assert res.shape == (self.nservos, self.nsteps_per_episode * self.nepisodes)
            return res
        elif self.type == ExperimentType.GENETIC:
            # Compile data for genetic experiment type
            ## Get the Fitnesses
            fitnesses = [generation.highest_fitness for generation in self.episodes]
            ## Get the servo data
            genarrays = []
            for generation in self.episodes:
                generation_data = generation.data_sorted_by_fitness
                genarrays.append(generation_data)
            res = np.array(genarrays).reshape((self.nservos, self.nsteps_per_episode, self.nnetworks, self.nepisodes))
            assert res.shape == (self.nservos, self.nsteps_per_episode, self.nnetworks, self.nepisodes)
            return fitnesses, res
        else:
            return None

def plot_random(experiment):
    """
    Plots the given experiment, which is assumed to be of type==ExperimentType.RANDOM
    """
    plt.title("Random Experiment's Servo Values Across Episodes")
    plt.xlabel("Step")
    plt.ylabel("Degrees")
    data = experiment.data
    for servo in range(data.shape[0]):
        plt.plot(data[servo, :])
    plt.show()

def plot_genetic(experiment):
    """
    Plots the given experiment, which is assumed to be of type==ExperimentType.GENETIC
    """
    fitnesses, servo_data = experiment.data
    plt.title("Each Generation's Best Fitness Value")
    plt.xlabel("Generation")
    plt.ylabel("Fitness Value")
    plt.plot(fitnesses)
    plt.show()

    plt.title("Servo Values from Best Network in Each Generation")
    plt.xlabel("Step (every {} steps is a generation)".format(experiment.nsteps_per_episode))
    plt.ylabel("Degrees")
    for servo in range(servo_data.shape[0]):
        # Since the data is sorted, we can assume the 0 index is the most fit network
        data = servo_data[servo, :, 0, :].reshape((experiment.nsteps_per_episode * experiment.nepisodes, ))
        plt.plot(data)
    plt.show()

if __name__ == "__main__":
    if len(sys.argv) != 2 and not os.path.exists("results.txt"):
        print("Need a path to results.txt file.")
        exit(1)
    elif len(sys.argv) == 2 and not os.path.exists(sys.argv[1]):
        print("{} does not exist.".format(sys.argv[1]))
        exit(1)

    if len(sys.argv) == 2:
        path = sys.argv[1]
    else:
        path = "results.txt"

    experiment = ExperimentResults(path)
    print(experiment)

    if experiment.type == ExperimentType.RANDOM:
        plot_random(experiment)
    elif experiment.type == ExperimentType.GENETIC:
        plot_genetic(experiment)
    else:
        raise Exception("Unsupported experiment type: {}".format(experiment.type))
