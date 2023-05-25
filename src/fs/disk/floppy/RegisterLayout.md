<h2>MSR byte:</h2>
<table>
   <thead>
      <tr>
         <th>Bit</th>
         <th>Name</th>
         <th>Description</th>
      </tr>
   </thead>
   <tbody>
      <tr>
         <td>7</td>
         <td>MRQ</td>
         <td>FIFO ready (1:yes, 0:no)</td>
      </tr>
      <tr>
         <td>6</td>
         <td>DIO</td>
         <td>Controller expecting read/write (1:write, 0:read)</td>
      </tr>
      <tr>
         <td>5</td>
         <td>NDMA</td>
         <td>Controller in DMA mode (1:noDMA, 0:DMA)</td>
      </tr>
      <tr>
         <td>4</td>
         <td>BUSY</td>
         <td>Controller executing a command (1=busy)</td>
      </tr>
      <tr>
         <td>3</td>
         <td>ACTD</td>
         <td>Drive D position/calibrated (1:yes, 0:no)</td>
      </tr>
      <tr>
         <td>2</td>
         <td>ACTC</td>
         <td>Drive C position/calibrated (1:yes, 0:no)</td>
      </tr>
      <tr>
         <td>1</td>
         <td>ACTB</td>
         <td>Drive B position/calibrated (1:yes, 0:no)</td>
      </tr>
      <tr>
         <td>0</td>
         <td>ACTA</td>
         <td>Drive A position/calibrated (1:yes, 0:no)</td>
      </tr>
   </tbody>
</table>
<h2>DOR byte: [write-only]</h2>
<table>
   <thead>
      <tr>
         <th>Bit</th>
         <th>Name</th>
         <th>Description</th>
      </tr>
   </thead>
   <tbody>
      <tr>
         <td>7</td>
         <td>MOTD</td>
         <td>Motor D control (1=on)</td>
      </tr>
      <tr>
         <td>6</td>
         <td>MOTC</td>
         <td>Motor C control (1=on)</td>
      </tr>
      <tr>
         <td>5</td>
         <td>MOTB</td>
         <td>Motor B control (1=on)</td>
      </tr>
      <tr>
         <td>4</td>
         <td>MOTA</td>
         <td>Motor A control (1=on)</td>
      </tr>
      <tr>
         <td>3</td>
         <td>DMA</td>
         <td>DMA line enables interrupts and DMA</td>
      </tr>
      <tr>
         <td>2</td>
         <td>NRST</td>
         <td>Controller enabled when 1 (not reset)</td>
      </tr>
      <tr>
         <td>1</td>
         <td>DR1</td>
         <td>Drive selection (bit 1) for current drive</td>
      </tr>
      <tr>
         <td>0</td>
         <td>DR0</td>
         <td>Drive selection (bit 0) for current drive</td>
      </tr>
   </tbody>
</table>
