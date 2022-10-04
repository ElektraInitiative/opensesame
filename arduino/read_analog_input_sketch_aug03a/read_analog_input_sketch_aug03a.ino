void setup() {
  Serial.begin(9600);
}

void loop() {
  Serial.print(analogRead(0), DEC);

  for (int i=1; i<12; ++i)  {
    Serial.write('\t');
    Serial.print(analogRead(i), DEC);
  }

  Serial.write('\r');

  delay(60000);
}
