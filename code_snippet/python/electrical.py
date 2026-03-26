import math
from typing import Tuple

class STM32F4ADC:
    def __init__(
        self,
        resistance_ain: float,
        resistance_adc: float,
        capacitance_adc: float,
        num_bits_resolution: int,
        freq_mhz_adc: int,
    ):
        """
        Initializes the ADC sampling time parameters.

        Args:
            resistor_ain (float): External analog input resistance.
            resistor_adc (float): Internal ADC input resistance.
            cap_adc (float): Internal ADC sampling capacitor.
            num_bits_resolution (int): ADC resolution in bits.
            num_sampling_cycles (float): Number of ADC sampling cycles.
            freq_mhz_adc (float): ADC clock frequency in MHz.
        """
        self._r_analog_in = resistance_ain
        self._r_adc = resistance_adc
        self._cap_adc = capacitance_adc
        self._num_bits_resolution = num_bits_resolution
        self._freq_mhz_adc = freq_mhz_adc
        self._cap_parasitic = 5 * pow(10, -12)  # Parasitic PCB Capacitance
        self._num_sampling_cycles = 0
        self._t_total_secs = 0
        self.refresh_params()

    @property
    def r_analog_in(self) -> float:
        return self._r_analog_in

    @r_analog_in.setter
    def r_analog_in(self, value: float):
        self._r_analog_in = value
        self.refresh_params()

    @property
    def r_adc(self) -> float:
        return self._r_adc

    @r_adc.setter
    def r_adc(self, value: float):
        self._r_adc = value
        self.refresh_params()

    @property
    def cap_adc(self) -> float:
        return self._cap_adc

    @cap_adc.setter
    def cap_adc(self, value: float):
        self._cap_adc = value
        self.refresh_params()

    @property
    def num_bits_resolution(self) -> int:
        return self._num_bits_resolution

    @num_bits_resolution.setter
    def num_bits_resolution(self, value: int):
        self._num_bits_resolution = value
        self.refresh_params()

    @property
    def num_sampling_cycles(self) -> float:
        return self._num_sampling_cycles

    @property
    def freq_mhz_adc(self) -> int:
        return self._freq_mhz_adc

    @freq_mhz_adc.setter
    def freq_mhz_adc(self, value: int):
        self._freq_mhz_adc = value
        self.refresh_params()

    def refresh_params(self):
        """
        implementation for equation 1 , from STM32F446xx datasheet,
        section 6.3.21 12-bit ADC characteristics
        """
        # Calculate the number of time constants required for 1/2 LSB error
        # ln(2^(N+1)) ensures error is below 1/2 LSB
        num_t_consts = math.log(pow(2, self._num_bits_resolution + 1), math.e)
        r_total = self._r_analog_in + self._r_adc
        c_total = self._cap_parasitic + self._cap_adc

        # Calculate total sampling time in seconds,
        # time charging sample-and-hold capacitor shared among the ADC channels
        t_total_secs = r_total * c_total * num_t_consts

        # Convert MHz to Hz
        f_hz_adc = self._freq_mhz_adc * pow(10, 6)
        self._num_sampling_cycles = (t_total_secs * f_hz_adc) + 0.5
        self._t_total_secs = t_total_secs

    def summary(self) -> str:
        """
        Dumps estimated parameters and relevant attributes for review.
        Returns:
            str: A formatted string containing the summary of ADC parameters.
        """
        return (
            f"--- ADC Parameter Estimation Summary ---\n"
            f"Input Resistance (R_AIN): {self._r_analog_in} Ohms\n"
            f"ADC Input Resistance (R_ADC): {self._r_adc} Ohms\n"
            f"ADC Sampling Capacitance (C_ADC): {self._cap_adc} Farads\n"
            f"Parasitic Capacitance (C_Parasitic): {self._cap_parasitic} Farads\n"
            f"ADC Resolution: {self._num_bits_resolution} bits\n"
            f"ADC Clock Frequency: {self._freq_mhz_adc} MHz\n"
            f"Estimated Number of Total Sample Time: {self._t_total_secs} Seconds\n"
            f"Estimated Number of Sampling Cycles: {self._num_sampling_cycles:.2f} cycles\n"
            f"----------------------------------------"
        )


class CapSoilSensorV2:
    c2 = 0.877 * pow(10, -6)
    r1 = 10 * pow(10, 3)
    r3 = 330
    r2 = 1600
    c3 = 0.98 * pow(10, -6)

    def __init__(self, p0: float, p1: float):
        self.z_r1_probe = p0
        self.z_r1_aout = p1
        # output frequency from TLC555 chip
        self.freq_max = 2.09 * pow(10, 6)
        self.freq_curr = 1.44 / ((self.r3 + 2 * self.r2) * self.c3)

    # reactance_cap = math.sqrt(pow(z_r1_probe, 2) - pow(r1, 2))
    # reactance_cap = math.sqrt(pow(z_r1_aout, 2) - pow(r1, 2))
    # cap_probe_min = 1 / (2 * math.pi * freq_max * reactance_cap)
    # cap_probe_curr = 1 / (2 * math.pi * freq_curr * reactance_cap)
    def _probe_cap(self, z_probe, freq_hz) -> float:
        reactance_cap = math.sqrt(pow(z_probe, 2) - pow(self.r1, 2))
        return 1 / (2 * math.pi * freq_hz * reactance_cap)

    def probe_capatitance(self) -> Tuple[float, float]:
        return [
            self._probe_cap(self.z_r1_probe, self.freq_curr),
            self._probe_cap(self.z_r1_aout, self.freq_curr),
        ]


# dry / air
sensordry = CapSoilSensorV2(170.6 * pow(10, 3), 74.8 * pow(10, 3))

# water / saturated
sensorwater = CapSoilSensorV2(63 * pow(10, 3), 76.7 * pow(10, 3))

# analog input impedance from capacitive soil sensors AOUT pin
adccfg0 = STM32F4ADC(
    resistance_ain=170.6 * pow(10, 3),
    resistance_adc=6 * pow(10, 3),
    capacitance_adc=4 * pow(10, -12),
    num_bits_resolution=12,
    freq_mhz_adc=30,
)

# adccfg0.num_bits_resolution = 10
# print(adccfg0.summary())
