# My Resume

This is the code that generates my resume.
You can have a look at <https://resume.lasernoises.com>.
I hope you can find the eastereggs.

It's an Axum webserver that serves PDFs generated on-the-fly using
[Laser-PDF](https://github.com/laser-pdf/laser-pdf), the PDF generation library I've built over the
last few years, while working at [Escola](https://www.escola.ch).
I deploy it to [Fly.io](https://fly.io) using a `Dockerfile` that generates an image that only
contains a statically linked binary.
